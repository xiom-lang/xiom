// XIOM — Stdlib Compilation Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Verifies that every stdlib .xi module compiles to IR with the current compiler.

use std::process::Command;
use std::path::Path;

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

/// Run xiomc --emit-ir on a single .xi file. Returns (success, stderr).
fn compile_stdlib_file(file_path: &str) -> (bool, String) {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", file_path])
        .current_dir(project_root())
        .output()
        .expect("failed to execute xiomc");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stderr)
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
    let modules = stdlib_modules();
    let total = modules.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<String> = Vec::new();

    let project_dir = project_root().display().to_string();

    for (name, path) in &modules {
        let full_path = format!("{}/{}", project_dir, path);
        if !Path::new(&full_path).exists() {
            failures.push(format!("  {}: FILE NOT FOUND at {}", name, full_path));
            failed += 1;
            continue;
        }

        let (success, stderr) = compile_stdlib_file(path);
        if success {
            passed += 1;
            eprintln!("  [OK]    xiom.{}", name);
        } else {
            failed += 1;
            // Extract first line of stderr for summary
            let first_error = stderr.lines()
                .find(|l| l.contains("error"))
                .unwrap_or(&stderr)
                .to_string();
            failures.push(format!("  [FAIL]  xiom.{} — {}", name, first_error));
            eprintln!("  [FAIL]  xiom.{}", name);
            eprintln!("{}", stderr);
        }
    }

    eprintln!("\n=== Stdlib Compilation Results ===");
    eprintln!("  Total:  {}", total);
    eprintln!("  Passed: {}", passed);
    eprintln!("  Failed: {}", failed);
    eprintln!("  Rate:   {:.1}%", (passed as f64 / total as f64) * 100.0);

    if !failures.is_empty() {
        eprintln!("\nFailures:");
        for f in &failures {
            eprintln!("{}", f);
        }
    }

    assert_eq!(failed, 0, "{} of {} stdlib modules failed to compile", failed, total);
}
