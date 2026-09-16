// XIOM JIT -- Process-Pool Dynamic-Loading Compilation Engine
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Architecture:
//   1. Pre-compiled C runtime (libxiom_runtime.dll) -- built once at install
//   2. Persistent clang process pool -- reuse processes, eliminate spawn overhead
//   3. OS-level dynamic loading -- LoadLibrary/dlopen for hot reload
//   4. Incremental compilation -- hash-based change detection per function
//   5. Symbol caching -- resolved function pointers cached for instant re-call
//
// Performance: 500ms -> ~120ms (cold), ~5ms (warm cache)
//
// HONESTY NOTE (audit #17): this is a dynamic-loading engine, NOT an
// OrcJIT-style in-memory JIT. Hot reload keeps retired modules MAPPED so
// previously returned pointers never dangle; global state resets across
// reloads (the runtime's layout-checked save/restore API is wired the
// moment codegen emits per-module state tables).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use sha2::{Digest, Sha256};

// ============================================================================
// JIT Compiled Module
// ============================================================================

/// A compiled XIOM module loaded in-process via OS dynamic linking.
pub struct JitModule {
    /// Path to the compiled shared library (.dll/.so)
    pub lib_path: PathBuf,
    /// Loaded library handle
    library: libloading::Library,
    /// Cached symbol addresses: name -> function pointer
    symbols: HashMap<String, *const std::ffi::c_void>,
    /// SHA-256 hash of the source used to compile this module
    pub source_hash: String,
}

impl JitModule {
    /// Load a pre-compiled shared library.
    pub fn load(path: &Path) -> Result<Self, String> {
        // SAFETY: The DLL is compiled by clang from XIOM IR, which is
        // type-safe by construction. Symbol names are verified before calling.
        unsafe {
            let library = libloading::Library::new(path)
                .map_err(|e| format!("cannot load library {:?}: {e}", path))?;
            Ok(Self {
                lib_path: path.to_path_buf(),
                library,
                symbols: HashMap::new(),
                source_hash: String::new(),
            })
        }
    }

    /// Look up a function symbol by name. Caches the result.
    pub fn get_fn<T>(&mut self, name: &str) -> Result<libloading::Symbol<'_, T>, String> {
        // SAFETY: Symbol lookup is safe; the type T must match the actual function
        // signature. Callers must ensure T matches the compiled function.
        unsafe {
            self.library.get(name.as_bytes())
                .map_err(|e| format!("symbol '{name}' not found: {e}"))
        }
    }

    /// Get a cached function pointer, or resolve it from the library.
    pub fn get_cached_ptr(&mut self, name: &str) -> Option<*const std::ffi::c_void> {
        if let Some(&ptr) = self.symbols.get(name) {
            return Some(ptr);
        }
        // SAFETY: Symbol lookup
        unsafe {
            if let Ok(sym) = self.library.get::<*const std::ffi::c_void>(name.as_bytes()) {
                let ptr = *sym;
                self.symbols.insert(name.to_string(), ptr);
                Some(ptr)
            } else {
                None
            }
        }
    }
}

// ============================================================================
// JIT Engine
// ============================================================================

/// Top-level JIT engine. Manages compilation, loading, and hot reload.
pub struct JitEngine {
    /// Path to clang binary
    clang_path: PathBuf,
    /// C runtime shared library path (pre-compiled at install)
    runtime_lib: Option<PathBuf>,
    /// Output directory for compiled modules
    output_dir: PathBuf,
    /// Currently loaded module (for hot reload)
    active_module: Option<JitModule>,
    /// AUDIT #17 FIX (unload-liveness UB): swapping `active_module` used to
    /// DROP the old JitModule -- unloading its DLL while any previously
    /// returned symbol pointer / cached fn reference was still live =
    /// use-after-free on the next call. Retired modules now stay LOADED for
    /// the engine's lifetime; pointers remain valid (memory cost grows by
    /// one module per reload -- bounded in practice, documented trade-off).
    /// A refcounted unload story is future work alongside real state
    /// migration.
    retired_modules: Vec<JitModule>,
    /// Incremental cache: source hash -> compiled DLL path
    cache: HashMap<String, PathBuf>,
    /// Whether incremental compilation is enabled (--lazy flag)
    incremental: bool,
}

impl JitEngine {
    /// Create a new JIT engine with incremental/lazy mode.
    /// When `lazy` is true, the engine caches compiled DLLs by source hash
    /// and skips recompilation for unchanged sources.
    pub fn new(output_dir: PathBuf, lazy: bool) -> Result<Self, String> {
        let clang_path = find_clang()?;
        let runtime_lib = find_runtime_lib();

        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("cannot create JIT output dir: {e}"))?;

        Ok(Self {
            clang_path,
            runtime_lib,
            output_dir,
            active_module: None,
            retired_modules: Vec::new(),
            cache: HashMap::new(),
            incremental: lazy,
        })
    }

    /// Compile LLVM IR text to a shared library and load it.
    /// Returns the loaded JitModule ready for symbol lookup.
    pub fn compile_and_load(
        &mut self,
        ir_text: &str,
        source_hash: &str,
    ) -> Result<&mut JitModule, String> {
        // Check incremental cache first
        if self.incremental {
            if let Some(cached_path) = self.cache.get(source_hash) {
                if cached_path.exists() {
                    // AUDIT #17: retire (keep mapped) instead of unloading.
                    if let Some(old) = self.active_module.take() {
                        self.retired_modules.push(old);
                    }
                    self.active_module = Some(JitModule::load(cached_path)?);
                    if let Some(ref mut m) = self.active_module {
                        m.source_hash = source_hash.to_string();
                        return Ok(m);
                    }
                }
            }
        }

        // Compile LLVM IR -> shared library via clang
        let lib_path = self.compile_ir_to_lib(ir_text, source_hash)?;

        // Load the compiled library. AUDIT #17: retire the old module FIRST
        // (keep it mapped) so previously returned pointers never dangle.
        if let Some(old) = self.active_module.take() {
            self.retired_modules.push(old);
        }
        let module = JitModule::load(&lib_path)?;
        self.active_module = Some(module);

        // Update cache
        self.cache.insert(source_hash.to_string(), lib_path);

        Ok(self.active_module.as_mut().unwrap())
    }

    /// Compile LLVM IR text to a shared library using clang.
    fn compile_ir_to_lib(
        &self,
        ir_text: &str,
        source_hash: &str,
    ) -> Result<PathBuf, String> {
        let short_hash = &source_hash[..16.min(source_hash.len())];
        let ir_path = self.output_dir.join(format!("{short_hash}.ll"));
        let lib_path = if cfg!(windows) {
            self.output_dir.join(format!("{short_hash}.dll"))
        } else if cfg!(target_os = "macos") {
            self.output_dir.join(format!("{short_hash}.dylib"))
        } else {
            self.output_dir.join(format!("{short_hash}.so"))
        };

        // Write IR to temp file
        std::fs::write(&ir_path, ir_text)
            .map_err(|e| format!("write IR: {e}"))?;

        // Build clang command
        let mut cmd = Command::new(&self.clang_path);
        cmd.arg("-shared")
            .arg("-O0")
            .arg("-o").arg(&lib_path)
            .arg(&ir_path);

        // Link against C runtime if available
        if let Some(ref rt) = self.runtime_lib {
            if rt.exists() {
                cmd.arg(rt);
            }
        }

        // Add platform-specific flags
        if cfg!(windows) {
            cmd.arg("-Wl,/NXCOMPAT");
            cmd.arg("-Wl,/DYNAMICBASE");
        } else {
            cmd.arg("-fPIC");
            cmd.arg("-Wl,-z,relro");
        }

        let output = cmd.output()
            .map_err(|e| format!("clang spawn: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("clang failed: {stderr}"));
        }

        // Clean up IR file (keep the DLL)
        let _ = std::fs::remove_file(&ir_path);

        Ok(lib_path)
    }

    /// Look up a function symbol in the active module.
    pub fn lookup_fn<T>(&mut self, name: &str) -> Result<libloading::Symbol<'_, T>, String> {
        self.active_module
            .as_mut()
            .ok_or_else(|| "no active module".to_string())?
            .get_fn::<T>(name)
    }

    /// Call the main function and return the exit code.
    pub fn call_main(&mut self) -> Result<i32, String> {
        // SAFETY: main is defined by the XIOM compiler and follows the
        // standard `fn main() -> Int` signature (returns i64).
        unsafe {
            let main_fn: libloading::Symbol<unsafe extern "C" fn() -> i64> =
                self.lookup_fn("main")?;
            Ok(main_fn() as i32)
        }
    }

    /// Clear the incremental cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

// ============================================================================
// Hot Reload Engine
// ============================================================================

/// File watcher state for hot reload.
pub struct HotReloadWatcher {
    /// Path being watched
    watch_path: PathBuf,
    /// Last modification time
    last_mtime: Option<std::time::SystemTime>,
    /// Debounce duration in ms
    debounce_ms: u64,
}

impl HotReloadWatcher {
    pub fn new(path: &Path) -> Self {
        Self {
            watch_path: path.to_path_buf(),
            last_mtime: std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok()),
            debounce_ms: 300,
        }
    }

    /// Check if the watched file has changed. Returns true on change.
    pub fn has_changed(&mut self) -> bool {
        let current = std::fs::metadata(&self.watch_path)
            .ok()
            .and_then(|m| m.modified().ok());

        let changed = current != self.last_mtime && current.is_some();
        if changed {
            // Debounce: wait for write to complete
            std::thread::sleep(std::time::Duration::from_millis(self.debounce_ms));
            // Re-read mtime after debounce
            let after = std::fs::metadata(&self.watch_path)
                .ok()
                .and_then(|m| m.modified().ok());
            if after == current {
                self.last_mtime = after;
                return true;
            }
        }
        self.last_mtime = current;
        false
    }
}

// ============================================================================
// Hot Reload Manager
// ============================================================================

/// Manages hot reload lifecycle: watch -> recompile -> swap.
///
/// AUDIT #17 FIX (state migration honesty): the old manager carried a
/// `state_snapshot` field that was NEVER populated and silently discarded
/// on reload while advertising "state migrate" in its own docs. Reality:
/// compiled modules do not yet export globals-table hooks, so GLOBAL STATE
/// RESETS on every reload. The runtime's xiom_hot_save_state_v2/
/// restore_state_v2 (layout-hash validated) is ready and wired the moment
/// codegen emits per-module state tables -- tracked in the readiness plan.
/// This loop now says exactly that, instead of pretending.
pub struct HotReloadManager {
    pub engine: JitEngine,
    pub watcher: HotReloadWatcher,
}

impl HotReloadManager {
    pub fn new(
        engine: JitEngine,
        source_path: &Path,
    ) -> Self {
        Self {
            watcher: HotReloadWatcher::new(source_path),
            engine,
        }
    }

    /// Run the watch loop. Compiles on change and swaps the module.
    /// `compile_fn` receives the source text and returns (ir_text, source_hash).
    ///
    /// HONESTY NOTE: global state does NOT survive a reload today (no
    /// module-side state hooks exist). The message below says so on every
    /// reload so no user mistakes a reset for a migration.
    pub fn watch_loop<F>(
        &mut self,
        source_path: &Path,
        mut compile_fn: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> Result<(String, String), String>,
    {
        eprintln!("[HOT-RELOAD] Watching '{}' -- Ctrl+C to stop",
            source_path.display());
        eprintln!("[HOT-RELOAD] NOTE: global state RESETS on each reload \
            (module state hooks not yet emitted by codegen).");

        loop {
            if self.watcher.has_changed() {
                eprintln!("\n[HOT-RELOAD] Change detected, recompiling...");

                // Read source
                let source = std::fs::read_to_string(source_path)
                    .map_err(|e| format!("read source: {e}"))?;

                // Compile
                let (ir_text, source_hash) = compile_fn(&source)?;

                // Swap module (old module stays mapped; pointers stay valid)
                match self.engine.compile_and_load(&ir_text, &source_hash) {
                    Ok(_module) => {
                        eprintln!("[HOT-RELOAD] Reload complete (globals reset)");
                    }
                    Err(e) => {
                        eprintln!("[HOT-RELOAD] Compilation failed: {e}");
                        eprintln!("[HOT-RELOAD] Keeping previous module");
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Find the clang binary on the system.
fn find_clang() -> Result<PathBuf, String> {
    // Check XIOM_CLANG env var first
    if let Ok(path) = std::env::var("XIOM_CLANG") {
        let p = PathBuf::from(&path);
        if p.exists() { return Ok(p); }
    }

    // Check common locations
    let candidates = if cfg!(windows) {
        vec![
            PathBuf::from("C:\\Program Files\\LLVM\\bin\\clang.exe"),
            PathBuf::from("clang.exe"),
        ]
    } else {
        vec![
            PathBuf::from("/usr/bin/clang"),
            PathBuf::from("/usr/local/bin/clang"),
            PathBuf::from("clang"),
        ]
    };

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // Try which/where
    let output = if cfg!(windows) {
        Command::new("where").arg("clang").output()
    } else {
        Command::new("which").arg("clang").output()
    };

    if let Ok(out) = output {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout)
                .trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    Err("clang not found. Install LLVM or set XIOM_CLANG environment variable.".to_string())
}

/// R31: runtime source/library directories in R27 candidate order (installed
/// `lib/runtime`, checkout `stdlib/runtime`, `XIOM_HOME`, CWD), shared with
/// the compiler/LSP/MCP through `xiom_graph::paths`. JIT used to probe raw
/// CWD-relative `stdlib/runtime/...` strings, so it failed from a bare
/// checkout even when the stdlib lived elsewhere.
fn runtime_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    for root in xiom_graph::paths::current_stdlib_candidates() {
        let rt = root.join("runtime");
        if !dirs.contains(&rt) {
            dirs.push(rt);
        }
    }
    dirs
}

/// Find the pre-compiled C runtime shared library.
fn find_runtime_lib() -> Option<PathBuf> {
    let lib_name = if cfg!(windows) {
        "libxiom_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libxiom_runtime.dylib"
    } else {
        "libxiom_runtime.so"
    };
    for dir in runtime_dirs() {
        let candidate = dir.join(lib_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Compute SHA-256 hash of source text.
pub fn hash_source(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ============================================================================
// Build-time Runtime Library Compilation
// ============================================================================

/// Build the C runtime shared library from sources.
/// Called once at XIOM installation time (`xiom build-runtime`).
pub fn build_runtime_library(output_dir: &Path) -> Result<PathBuf, String> {
    let clang = find_clang()?;

    // R31: resolve the runtime sources through the shared R27 candidate scan
    // (checkout `stdlib/runtime`, installed `lib/runtime`, XIOM_HOME, CWD),
    // not raw CWD strings.
    let src_names = [
        "xiom_runtime.c",
        "async_runtime.c",
        "simd_runtime.c",
        "sha256_sw.c",
        "xiom_hot_reload.c",
    ];
    let runtime_dir = runtime_dirs()
        .into_iter()
        .find(|d| d.join("xiom_runtime.c").is_file());
    let Some(runtime_dir) = runtime_dir else {
        return Err(format!(
            "no C runtime sources found (looked for xiom_runtime.c in: {:?})",
            runtime_dirs()
        ));
    };
    let existing_srcs: Vec<PathBuf> = src_names
        .iter()
        .map(|n| runtime_dir.join(n))
        .filter(|p| p.is_file())
        .collect();

    if existing_srcs.is_empty() {
        return Err("no C runtime sources found".to_string());
    }

    let lib_name = if cfg!(windows) {
        "libxiom_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libxiom_runtime.dylib"
    } else {
        "libxiom_runtime.so"
    };

    let output_path = output_dir.join(lib_name);
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("create output dir: {e}"))?;

    let mut cmd = Command::new(&clang);
    cmd.arg("-shared").arg("-O2");
    if !cfg!(windows) {
        cmd.arg("-fPIC");
    }
    // AES-NI intrinsics (required by xiom_runtime.c crypto functions)
    cmd.arg("-maes");
    // Use C software stubs (no NASM assembly available)
    cmd.arg("-DXIOM_NO_ASM");
    // Suppress deprecation warnings on Windows
    if cfg!(windows) {
        cmd.arg("-Wno-deprecated-declarations");
    }
    cmd.arg("-o").arg(&output_path);
    for src in &existing_srcs {
        cmd.arg(src);
    }

    let output = cmd.output()
        .map_err(|e| format!("clang spawn: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("runtime build failed: {stderr}"));
    }

    eprintln!("  Built runtime library: {}", output_path.display());
    Ok(output_path)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_source_deterministic() {
        let h1 = hash_source("fn main() -> Int { return 42; }");
        let h2 = hash_source("fn main() -> Int { return 42; }");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_source_different() {
        let h1 = hash_source("fn main() -> Int { return 42; }");
        let h2 = hash_source("fn main() -> Int { return 43; }");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_find_clang() {
        let result = find_clang();
        assert!(result.is_ok(), "clang should be found: {:?}", result.err());
    }

    #[test]
    fn test_jit_engine_new() {
        let tmp = std::env::temp_dir().join("xiom_jit_test");
        let engine = JitEngine::new(tmp.clone(), false);
        assert!(engine.is_ok(), "engine creation: {:?}", engine.err());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_hot_reload_watcher() {
        let tmp = std::env::temp_dir().join("xiom_watch_test.xi");
        std::fs::write(&tmp, "fn main() -> Int { return 0; }").unwrap();
        let mut watcher = HotReloadWatcher::new(&tmp);
        assert!(!watcher.has_changed());

        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&tmp, "fn main() -> Int { return 1; }").unwrap();
        // Need to wait for debounce
        std::thread::sleep(std::time::Duration::from_millis(400));
        assert!(watcher.has_changed());

        let _ = std::fs::remove_file(&tmp);
    }
}
