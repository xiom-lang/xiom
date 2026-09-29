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

use std::path::PathBuf;
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
