// XIOM Codegen -- JIT execution via shared library loading (M10)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Pipeline: .xi source -> AOT -> shared library (.dll/.so) -> dlopen -> call main() -> result

use std::path::PathBuf;

/// Result of JIT execution: exit code and captured stdout/stderr.
pub struct JitResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// JIT-compile and execute a XIOM source file.
/// This compiles to a shared library, loads it, and calls main().
pub fn jit_compile_and_run(source_path: &str) -> Result<JitResult, String> {
    // For now, delegate to AOT path with shared library target.
    // The actual JIT happens in the xiom crate which has access to the compiler.
    Err("JIT execution requires the xiom crate integration".to_string())
}

/// Path for the JIT cache directory.
pub fn jit_cache_dir() -> PathBuf {
    dirs_next().unwrap_or_else(|| std::env::temp_dir().join(".xiom").join("jit"))
}

fn dirs_next() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        Some(PathBuf::from(home).join(".xiom").join("jit"))
    } else {
        Some(std::env::temp_dir().join("xiom_jit"))
    }
}
