// XIOM — True JIT Execution via shared library loading (M10)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Pipeline: .xi source → AOT → .dll/.so → load in-process → call main() → result

use std::path::PathBuf;
use crate::CompileConfig;

/// JIT-compile XIOM source to a shared library, load it, and call main().
/// Returns the exit code from main(), or an error message.
pub fn jit_execute(source: &str) -> Result<i32, String> {
    let tmp_dir = std::env::temp_dir().join("xiom_jit");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("cannot create temp dir: {e}"))?;

    let tmp_src = tmp_dir.join("_jit.xi");
    let tmp_out = if cfg!(windows) {
        tmp_dir.join("_jit.dll")
    } else if cfg!(target_os = "macos") {
        tmp_dir.join("_jit.dylib")
    } else {
        tmp_dir.join("_jit.so")
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
    unsafe {
        let lib = libloading::Library::new(&tmp_out)
            .map_err(|e| format!("cannot load library: {e}"))?;

        let main_fn: libloading::Symbol<unsafe extern "C" fn() -> i64> = lib
            .get(b"main")
            .map_err(|e| format!("main not exported: {e}"))?;

        let exit_code = main_fn();
        Ok(exit_code as i32)
    }
}

/// JIT-execute with wrapped source (applies implicit main if needed).
pub fn jit_run_wrapped(raw_source: &str) -> Result<i32, String> {
    let wrapped = crate::implicit_main::wrap_implicit_main(raw_source);
    jit_execute(&wrapped)
}

/// Content-hash based script cache.
pub fn script_cache_get(source: &str) -> Option<PathBuf> {
    let cache_dir = jit_cache_dir();
    let hash = hash_source(source);
    let cache_file = cache_dir.join(format!("{:x}", hash));
    if cfg!(windows) {
        let exe = cache_file.with_extension("exe");
        if exe.exists() { return Some(exe); }
    }
    if cache_file.exists() { return Some(cache_file); }
    None
}

pub fn script_cache_put(source: &str, binary: &PathBuf) {
    let cache_dir = jit_cache_dir();
    std::fs::create_dir_all(&cache_dir).ok();
    let hash = hash_source(source);
    let cache_file = cache_dir.join(format!("{:x}", hash));
    let target = if cfg!(windows) { cache_file.with_extension("exe") } else { cache_file };
    std::fs::copy(binary, &target).ok();
}

pub fn jit_cache_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        PathBuf::from(home).join(".xiom").join("jit")
    } else {
        std::env::temp_dir().join("xiom_jit")
    }
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

fn hash_source(source: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
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
}
