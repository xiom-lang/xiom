// m281 (REL-20261012-2051-bindings-2 B-23, 2026-10-13): the bare `null`
// pointer literal must marshal to C as a true NULL pointer. Pre-fix the
// Ident arg classified as not-pointer-valued and the materialization path
// emitted `alloca i8 + trunc/store` -- C received the ADDRESS OF A STACK
// BYTE (varying per run; dx12 CreateCommittedResource(pOptimizedClearValue=
// null) -> E_INVALIDARG). The lock asserts all four paths (direct, via an
// XIOM fn param, from-C zero, from-C re-passed) read 0, and that the IR
// materializes the null as inttoptr, never as an i8 stack slot.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

#[test]
fn m281_null_literal_to_c() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m281_null_literal_to_c");
    let dir = std::env::temp_dir().join(format!("xiom_m281_null_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let main_xi = dir.join("main.xi");
    let host_c = dir.join("host.c");
    std::fs::copy(fixture.join("main.xi"), &main_xi).expect("fixture main.xi must exist");
    std::fs::copy(fixture.join("host.c"), &host_c).expect("fixture host.c must exist");

    // IR lock: the direct-null call site must be `inttoptr i64 0 to i8*`
    // (or pass the literal `null`), never an alloca'd byte.
    let ir = Command::new(xiom_bin())
        .arg(&main_xi)
        .arg("--emit-ir")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        ir.status.success(),
        "m281 null fixture must emit IR; exit={:?}\nstderr:\n{}",
        ir.status.code(),
        String::from_utf8_lossy(&ir.stderr)
    );
    let ir_text = String::from_utf8_lossy(&ir.stdout);
    assert!(
        ir_text.contains("inttoptr i64 0 to i8*")
            || ir_text.contains("xs_ptr_val(i8* null)"),
        "the null literal must lower to a null pointer (no stack byte); IR:\n{ir_text}"
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
        "m281 null fixture must compile; exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&out_bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", out_bin.display()));
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "null must cross to C as 0 on all four paths; exit={:?}\nstdout:\n{stdout}\nstderr:\n{}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        stdout.contains("direct=0") && stdout.contains("via_fn=0")
            && stdout.contains("from_c=0") && stdout.contains("via_var=0"),
        "all four null paths must read 0; stdout:\n{stdout}"
    );
}
