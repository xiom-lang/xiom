// XIOM -- True JIT Execution via shared library loading (M10)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Pipeline: .xi source -> AOT -> .dll/.so -> load in-process -> call main() -> result

use std::path::PathBuf;
use crate::CompileConfig;

/// JIT-compile XIOM source to a shared library, load it, and call main().
/// Returns the exit code from main(), or an error message.
pub fn jit_execute(source: &str) -> Result<i32, String> {
    // Unique temp dir per invocation -- parallel JIT calls (e.g. tests) must
    // not collide on a shared _jit.xi/_jit.dll path. Pid alone was not
    // enough: pid reuse plus a stale directory from a crashed run could hit
    // the same paths, so mix in the time+counter suffix. The directory is
    // removed best-effort after the library is dropped (Windows releases the
    // file lock on drop).
    let tmp_dir = std::env::temp_dir().join(format!(
        "xiom_jit_{}_{:x}",
        std::process::id(),
        rand_suffix()
    ));
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("cannot create temp dir: {e}"))?;

    let tmp_src = tmp_dir.join(format!("_jit_{:x}.xi", rand_suffix()));
    let tmp_out = if cfg!(windows) {
        tmp_dir.join(format!("_jit_{:x}.dll", rand_suffix()))
    } else if cfg!(target_os = "macos") {
        tmp_dir.join(format!("_jit_{:x}.dylib", rand_suffix()))
    } else {
        tmp_dir.join(format!("_jit_{:x}.so", rand_suffix()))
    };

    std::fs::write(&tmp_src, source).map_err(|e| format!("cannot write source: {e}"))?;

    // Compile to shared library
    let config = CompileConfig {
        shared_lib: true,
        output_file: Some(tmp_out.to_string_lossy().to_string()),
        ..CompileConfig::default()
    };
    crate::compile(&config, &[tmp_src.to_string_lossy().to_string()])
        .map_err(|e| format!("compile failed: {:?}", e))?;

    // SAFETY: The compiled DLL is loaded via libloading. The main() symbol
    // is verified to exist before calling. The DLL is loaded into the current
    // process and must be a valid XIOM-compiled shared library with a
    // `fn main() -> Int` entry point. Memory safety is enforced by the
    // XIOM compiler's type system and borrow checker on the source code.
    let exit_code = unsafe {
        let lib = libloading::Library::new(&tmp_out)
            .map_err(|e| format!("cannot load library: {e}"))?;

        let main_fn: libloading::Symbol<unsafe extern "C" fn() -> i64> = lib
            .get(b"main")
            .map_err(|e| format!("main not exported: {e}"))?;

        main_fn()
    };
    // `lib` has been dropped (unloaded) above, so the artifact lock is gone.
    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(exit_code as i32)
}

/// Time+counter-based suffix so concurrent compilations (and concurrent
/// driver processes sharing a temp dir) use distinct file names.
pub fn rand_suffix() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let ctr = COUNTER.fetch_add(1, Ordering::Relaxed);
    nanos ^ (ctr.wrapping_mul(0x9E3779B97F4A7C15))
}

/// JIT-execute with wrapped source (applies implicit main if needed).
pub fn jit_run_wrapped(raw_source: &str) -> Result<i32, String> {
    let wrapped = crate::implicit_main::wrap_implicit_main(raw_source);
    jit_execute(&wrapped)
}

/// Content-hash based script cache using SHA-256 for strong identity.
/// Cached binaries live in `~/.xiom/jit/<sha256hex>`.
/// Returns the cached binary path if it exists and is valid.
/// R48: identity of THIS compiler build for the script cache. The cache used
/// to be keyed by the source hash alone, so a freshly built compiler silently
/// served binaries produced by the previous build (playground verified the
/// v0.60.1 -> v0.61.0 crossover). Version + OS/arch + pointer width separates
/// any two builds that can emit different code for the same source.
pub fn compiler_cache_identity() -> String {
    format!(
        "{}|{}-{}|{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        usize::BITS
    )
}

/// R51 (playground audit §19.1): the EFFECTIVE optimization level for the
/// script cache -- an explicit `--opt-level`, else the compiler default
/// (-O2 debug, -O3 release). Part of the cache key so a binary built at one
/// level is never served for another.
pub fn effective_opt_level(config_level: Option<u8>, release: bool) -> u8 {
    config_level.unwrap_or(if release { 3 } else { 2 })
}

/// Cache key: source hash salted with the compiler build identity AND the
/// effective optimization level (R51: the level changes the emitted code,
/// so it must separate cache entries).
fn cache_key(source: &str, opt_level: u8) -> String {
    hash_source(&format!(
        "xiom-cache-v3|{}|opt={}|{}",
        compiler_cache_identity(),
        opt_level,
        source
    ))
}

/// Look up a cached script binary (compile-time `opt_level` 0..=3).
pub fn script_cache_get(source: &str, opt_level: u8) -> Option<PathBuf> {
    let cache_dir = jit_cache_dir();
    let hash = cache_key(source, opt_level);
    let cache_file = cache_dir.join(&hash);
    if cfg!(windows) {
        let exe = cache_file.with_extension("exe");
        if exe.exists() { return Some(exe); }
    }
    if cache_file.exists() { return Some(cache_file); }
    None
}

/// Store a compiled binary in the script cache keyed by SHA-256 of the source
/// AND the compiler build identity AND the effective optimization level.
pub fn script_cache_put(source: &str, binary: &PathBuf, opt_level: u8) {
    let cache_dir = jit_cache_dir();
    std::fs::create_dir_all(&cache_dir).ok();
    let hash = cache_key(source, opt_level);
    let cache_file = cache_dir.join(&hash);
    let target = if cfg!(windows) { cache_file.with_extension("exe") } else { cache_file };
    std::fs::copy(binary, &target).ok();
}

/// Check if a cached binary exists for the given source (without returning path).
pub fn script_cache_has(source: &str, opt_level: u8) -> bool {
    script_cache_get(source, opt_level).is_some()
}

pub fn jit_cache_dir() -> PathBuf {
    // R51 (playground audit §19.3): treat an EMPTY HOME/USERPROFILE as
    // unavailable -- an empty value produced a cwd-relative ".xiom/jit"
    // cache (often unwritable, and the playground's sandbox has no HOME)
    // instead of the temp fallback that keeps the cache working.
    for key in ["HOME", "USERPROFILE"] {
        if let Ok(dir) = std::env::var(key) {
            let dir = dir.trim();
            if !dir.is_empty() {
                let candidate = PathBuf::from(dir).join(".xiom").join("jit");
                // R63 (playground repro pack §4): a SET but UNWRITABLE home
                // (readonly HOME, sandbox) used to disable the script cache
                // silently -- every `xiom run` recompiled. Probe the
                // directory by creating it and fall back to temp on failure.
                if std::fs::create_dir_all(&candidate).is_ok() {
                    return candidate;
                }
                break;
            }
        }
    }
    std::env::temp_dir().join("xiom_jit")
}

const MAX_CACHE_SIZE: u64 = 100 * 1024 * 1024;

pub fn cache_evict_if_needed() {
    let cache_dir = jit_cache_dir();
    if !cache_dir.is_dir() { return; }

    let mut entries: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    let mut total_size: u64 = 0;

    if let Ok(read_dir) = std::fs::read_dir(&cache_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    let size = meta.len();
                    let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    total_size += size;
                    entries.push((mtime, path));
                }
            }
        }
    }
    if total_size <= MAX_CACHE_SIZE { return; }
    entries.sort_by_key(|(mtime, _)| *mtime);
    for (_, path) in &entries {
        if total_size <= MAX_CACHE_SIZE { break; }
        if let Ok(meta) = path.metadata() {
            total_size = total_size.saturating_sub(meta.len());
            let _ = std::fs::remove_file(path);
        }
    }
}

pub fn cache_clean() -> Result<u64, String> {
    let cache_dir = jit_cache_dir();
    if !cache_dir.is_dir() { return Ok(0); }
    let mut removed = 0u64;
    let mut total_bytes = 0u64;
    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() { total_bytes += meta.len(); }
                if std::fs::remove_file(&path).is_ok() { removed += 1; }
            }
        }
    }
    eprintln!("  Cleaned {removed} cached scripts ({:.1} MB)", total_bytes as f64 / 1_048_576.0);
    Ok(removed)
}

/// Hash source text with SHA-256 for strong cache identity.
/// Returns the hex digest as a string for use as a cache key/filename.
fn hash_source(source: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_execute_returns_exit_code() {
        let src = "fn main() -> Int { return 42; }\n";
        let result = jit_execute(src);
        assert!(result.is_ok(), "JIT should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), 42, "JIT should return 42");
    }

    #[test]
    fn test_jit_execute_with_implicit_main() {
        let src = "io.println(\"jit test\");\n";
        let result = jit_run_wrapped(src);
        assert!(result.is_ok(), "JIT with implicit main should succeed: {:?}", result.err());
    }

    #[test]
    fn test_jit_execute_void_main() {
        let src = "fn main() { }\n";
        let result = jit_execute(src);
        assert!(result.is_ok(), "void main should succeed: {:?}", result.err());
    }

    /// R48: the script cache must separate compiler builds. Old entries keyed
    /// by source hash alone made a new build serve the previous build's
    /// binaries (playground stale-cache hazard).
    /// R51: the effective optimization level is part of the key too.
    #[test]
    fn script_cache_key_includes_compiler_build_identity() {
        assert_eq!(cache_key("src", 2), cache_key("src", 2));
        assert_ne!(cache_key("src", 2), cache_key("other", 2));
        assert_ne!(
            cache_key("src", 0),
            cache_key("src", 2),
            "different opt levels must not share a cache entry"
        );
        assert!(
            compiler_cache_identity().contains(env!("CARGO_PKG_VERSION")),
            "identity must carry the compiler version: {}",
            compiler_cache_identity()
        );
        assert!(cache_key("src", 2).starts_with(|c: char| c.is_ascii_hexdigit()));
        assert_eq!(effective_opt_level(None, false), 2);
        assert_eq!(effective_opt_level(None, true), 3);
        assert_eq!(effective_opt_level(Some(0), true), 0);
    }
}
