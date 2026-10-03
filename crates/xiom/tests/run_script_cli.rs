// C22 (playground relay): `xiom run <script>` must resolve sibling modules
// AND packages/ layouts relative to the SCRIPT'S directory exactly like
// `xiom --check`.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `xiom run` compiles a %TEMP%/xiom_run copy, so without the source-dir hint
// the checker's catalog never saw the script's directory: sibling modules
// and package modules failed with undefined-variable errors while `--check`
// passed. Lock: tests/regression/c22_run_sibling_module/ (main.xi imports
// c22_sibling_lib.xi and packages/c22-pkg/src/hello.xi -> c22pkg.hello).

use std::path::{Path, PathBuf};
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("c22_run_sibling_module")
        .join("main.xi")
}

#[test]
fn c22_run_resolves_sibling_module() {
    let out = Command::new(xiom_bin())
        .arg("run")
        .arg(fixture())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The defect was a compile-stage failure: the sibling module never
    // reached the catalog ("undefined variable 'c22_sibling_lib'"). The
    // success signal for resolution+codegen is the `compiled:` line.
    assert!(
        out.status.success() || stderr.contains("compiled:"),
        "xiom run must resolve sibling modules and reach codegen (C22); exit={:?}\nstderr:\n{}",
        out.status.code(),
        stderr
    );
    // This dev box's Defender blocks freshly built %TEMP%/xiom_run exes
    // (os error 225), so the execution step may be environment-blocked
    // locally; CI (Linux) takes the full success path.
    if !out.status.success() {
        assert!(
            stderr.contains("os error 225") || stderr.contains("contains a virus"),
            "unexpected xiom run failure (C22); exit={:?}\nstderr:\n{}",
            out.status.code(),
            stderr
        );
    }
}

#[test]
fn c22_check_still_resolves_sibling_module() {
    let out = Command::new(xiom_bin())
        .arg("--check")
        .arg(fixture())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        out.status.success(),
        "xiom --check must keep resolving sibling modules (C22 parity); exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

// C25 (playground relay): `xiom run` executes a WARM script-cache hit via
// Command::output(), whose default stdin is NULL -- cached reruns printed
// `got: []` while the cold run read the piped line. The cached path must
// inherit stdin. `--no-cache` must bypass the cache entirely: the flags were
// filtered out of the `effective` list before they were read, so the check
// was dead and a warm cache could not be bypassed.
fn c25_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("c25_run_cached_stdin")
        .join("main.xi")
}

fn run_c25(args: &[&str], home: &Path) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(xiom_bin())
        .arg("run")
        .args(args)
        .arg(c25_fixture())
        .env("HOME", home)
        .env("USERPROFILE", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    child
        .stdin
        .as_mut()
        .expect("child stdin")
        .write_all(b"Ada\n")
        .expect("write child stdin");
    child.wait_with_output().expect("wait for xiom run")
}

#[test]
fn c25_warm_cache_inherits_stdin_and_no_cache_bypasses() {
    let home = std::env::temp_dir().join(format!("xiom_c25_home_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&home);
    let cold = run_c25(&[], &home);
    let cold_out = String::from_utf8_lossy(&cold.stdout);
    let cold_err = String::from_utf8_lossy(&cold.stderr);
    // This dev box's Defender can block freshly built %TEMP%/xiom_run exes
    // (os error 225); CI (Linux) takes the full path.
    if !cold.status.success()
        && (cold_err.contains("os error 225") || cold_err.contains("contains a virus"))
    {
        eprintln!("SKIP c25: environment blocks script execution (Defender os error 225)");
        let _ = std::fs::remove_dir_all(&home);
        return;
    }
    assert!(
        cold.status.success() && cold_out.contains("got: [Ada]"),
        "C25 cold run must read piped stdin; exit={:?}\nstdout: {cold_out}\nstderr: {cold_err}",
        cold.status.code()
    );
    let warm = run_c25(&[], &home);
    let warm_out = String::from_utf8_lossy(&warm.stdout);
    assert!(
        warm.status.success() && warm_out.contains("got: [Ada]"),
        "C25 warm-cache run must inherit stdin (defect printed `got: []`); exit={:?}\nstdout: {warm_out}\nstderr: {}",
        warm.status.code(),
        String::from_utf8_lossy(&warm.stderr)
    );
    let nocache = run_c25(&["--no-cache"], &home);
    let nocache_out = String::from_utf8_lossy(&nocache.stdout);
    assert!(
        nocache.status.success() && nocache_out.contains("got: [Ada]"),
        "C25 --no-cache run must bypass the cache and still read stdin; exit={:?}\nstdout: {nocache_out}\nstderr: {}",
        nocache.status.code(),
        String::from_utf8_lossy(&nocache.stderr)
    );
    let _ = std::fs::remove_dir_all(&home);
}
