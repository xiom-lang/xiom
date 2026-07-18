// XIOM — Stdlib Compilation Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Verifies that every stdlib .xi module compiles to IR with the current compiler.

use std::process::Command;
use std::path::Path;
use std::io::Write;
use std::fs;

fn xiomc_path() -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join("xiomc.exe");
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join("xiomc.exe");
    }
    path.to_str().unwrap().to_string()
}

fn project_root() -> &'static Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .to_path_buf()
    }).as_path()
}

/// All stdlib .xi files to compile (module name → relative path from project root)
fn stdlib_modules() -> Vec<(&'static str, &'static str)> {
    vec![
        ("core",        "stdlib/xiom/core.xi"),
        ("array",       "stdlib/xiom/array.xi"),
        ("string",      "stdlib/xiom/string.xi"),
        ("collections", "stdlib/xiom/collections.xi"),
        ("io",          "stdlib/xiom/io.xi"),
        ("fmt",         "stdlib/xiom/fmt.xi"),
        ("iter",        "stdlib/xiom/iter.xi"),
        ("math",        "stdlib/xiom/math.xi"),
        ("num",         "stdlib/xiom/num.xi"),
        ("cmp",         "stdlib/xiom/cmp.xi"),
        ("hash",        "stdlib/xiom/hash.xi"),
        ("mem",         "stdlib/xiom/mem.xi"),
        ("ptr",         "stdlib/xiom/ptr.xi"),
        ("char",        "stdlib/xiom/char.xi"),
        ("path",        "stdlib/xiom/path.xi"),
        ("convert",     "stdlib/xiom/convert.xi"),
        ("ffi",         "stdlib/xiom/ffi.xi"),
        ("sync",        "stdlib/xiom/sync.xi"),
        ("thread",      "stdlib/xiom/thread.xi"),
        ("async",       "stdlib/xiom/async.xi"),
        ("net",         "stdlib/xiom/net.xi"),
        ("os",          "stdlib/xiom/os.xi"),
        ("time",        "stdlib/xiom/time.xi"),
        ("test",        "stdlib/xiom/test.xi"),
        ("bench",       "stdlib/xiom/bench.xi"),
        ("log",         "stdlib/xiom/log.xi"),
        ("serialize",   "stdlib/xiom/serialize.xi"),
        ("crypto",      "stdlib/xiom/crypto.xi"),
        ("regex",       "stdlib/xiom/regex.xi"),
        ("rand",        "stdlib/xiom/rand.xi"),
        ("encoding",    "stdlib/xiom/encoding.xi"),
        ("compress",    "stdlib/xiom/compress.xi"),
        ("contracts",   "stdlib/xiom/contracts.xi"),
        ("reflect",     "stdlib/xiom/reflect.xi"),
        ("cell",        "stdlib/xiom/cell.xi"),
        ("rc",          "stdlib/xiom/rc.xi"),
        ("alloc",       "stdlib/xiom/alloc.xi"),
        ("env",         "stdlib/xiom/env.xi"),
        ("error",       "stdlib/xiom/error.xi"),
    ]
}

#[test]
fn stdlib_all_modules_compile_to_ir() {
    // Compile ALL stdlib modules TOGETHER in a single compilation unit
    // so cross-module references (e.g. core.xi::from_cstring used by
    // string.xi) are resolved. Isolated per-file compilation was failing
    // because modules depend on each other — not because of checker bugs.
    let modules = stdlib_modules();
    let total = modules.len();
    let project_dir = project_root().display().to_string();

    // Build a synthetic program that imports all stdlib modules.
    // We use `use xiom.XXX;` to bring each module into scope, then
    // a minimal main() to ensure the linker can generate code.
    let mut program = String::new();
    for (name, _path) in &modules {
        program.push_str(&format!("use xiom.{};\n", name));
    }
    program.push_str("fn main() -> Int { return 0; }\n");

    // Write to a temp file
    let tmp_dir = std::env::temp_dir();
    let tmp_file = tmp_dir.join("xiom_stdlib_full_test.xi");
    let mut f = fs::File::create(&tmp_file).expect("create temp file");
    f.write_all(program.as_bytes()).expect("write temp file");
    let tmp_path = tmp_file.to_str().unwrap().to_string();

    // Compile with --emit-ir (checker pass is required)
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", &tmp_path])
        .current_dir(&project_dir)
        .output()
        .expect("failed to execute xiomc");

    let _ = fs::remove_file(&tmp_file);

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "stdlib full compilation failed ({} modules together):\n{}",
        total, stderr
    );
}
