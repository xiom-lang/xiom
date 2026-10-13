// m282 (REL-20261011-2053-engine + REL-20261011-1627/1402-xvector,
// 2026-10-13): a fn-typed VALUE cast to a raw pointer must hand over the RAW
// CODE address, not the closure ENV handle. Pre-fix `f as *UInt8` inttoptr'd
// the env pointer: the xiom.thread.spawn runtime trampoline called the heap
// env address and AV'd (0xC0000005) before any worker body ran; the same
// cast through a local (`let g = f; g as *UInt8`) failed identically.
// The fixture passes the pointer to C (host.c call6), which invokes it; the
// bare-name shape stays green as a control.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

#[test]
fn m282_fn_env_to_raw_ptr() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m282_fn_env_raw_ptr");
    let dir = std::env::temp_dir().join(format!("xiom_m282_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let main_xi = dir.join("main.xi");
    let host_c = dir.join("host.c");
    std::fs::copy(fixture.join("main.xi"), &main_xi).expect("fixture main.xi must exist");
    std::fs::copy(fixture.join("host.c"), &host_c).expect("fixture host.c must exist");

    // IR lock: the cast must load env field 1 (raw code) -- the pre-fix
    // lowering inttoptr'd the env handle with no field load.
    let ir = Command::new(xiom_bin())
        .arg(&main_xi)
        .arg("--emit-ir")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        ir.status.success(),
        "m282 fixture must emit IR; exit={:?}\nstderr:\n{}",
        ir.status.code(),
        String::from_utf8_lossy(&ir.stderr)
    );
    let ir_text = String::from_utf8_lossy(&ir.stdout);
    assert!(
        ir_text.contains("gep i64, i64*") || ir_text.contains("getelementptr i64, i64*"),
        "the fn-env cast must load env field 1 (raw code); IR:\n{ir_text}"
    );

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
        "m282 fixture must compile; exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&out_bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", out_bin.display()));
    assert_eq!(
        run.status.code(),
        Some(0),
        "fn-env->raw-pointer casts must yield callable code (pre-fix exit 2); exit={:?}\nstdout:\n{}\nstderr:\n{}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}
