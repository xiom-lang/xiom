// m263 (REL-20261011-1055 B-13 + REL-20261010-1904, 2026-10-11): fn-VALUE
// C-boundary ABI lock. The fixture relays fn-typed params to C callbacks
// (direct, through a local, through a struct field, `as Int`) and calls a
// Float64-returning C function through a CAST fn-pointer. Pre-fix the param
// paths handed C the heap env pointer (the callback received a data address
// -> 0) and the cast call read the integer register for a double return.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

#[test]
fn m263_fn_value_c_abi() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m263_fn_value_c_abi");
    let dir = std::env::temp_dir().join(format!("xiom_m263_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let main_xi = dir.join("main.xi");
    let host_c = dir.join("host.c");
    std::fs::copy(fixture.join("main.xi"), &main_xi).expect("fixture main.xi must exist");
    std::fs::copy(fixture.join("host.c"), &host_c).expect("fixture host.c must exist");
    let out_bin = dir.join("probe_out.exe");
    let out = Command::new(xiom_bin())
        .arg(&main_xi)
        .arg("--c-source")
        .arg(&host_c)
        .arg("-o")
        .arg(&out_bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        out.status.success(),
        "m263 fixture must compile; exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&out_bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", out_bin.display()));
    assert_eq!(
        run.status.code(),
        Some(0),
        "m263 probe must exit 0\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("m263 ok"),
        "probe marker missing; stdout:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
