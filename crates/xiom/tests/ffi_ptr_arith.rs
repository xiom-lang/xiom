// m279 (REL-20261011-1512-bindings-2 B-17 + REL-20261011-1437-bindings-3
// B-16, 2026-10-13): raw-pointer arithmetic + ffi.ptr_write C-boundary lock.
// Pre-fix `base + N` on an owned buffer pointer lowered as Str concatenation
// (a fresh concat buffer -- wild addresses), so ffi.ptr_write_* at base+4/+8
// landed in unowned memory (ptr_write_sum=11 vs byte_index_sum=66) and a
// wild dst pointer caused nondeterministic heap corruption in xiom.sdl3.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

#[test]
fn m279_ffi_ptr_arith() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m279_ffi_ptr_arith");
    let dir = std::env::temp_dir().join(format!("xiom_m279_{}", std::process::id()));
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
        "m279 fixture must compile; exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&out_bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", out_bin.display()));
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "pointer arithmetic must be exact and both sums 66; exit={:?}\nstdout:\n{stdout}\nstderr:\n{}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        stdout.contains("ptr_write_sum=66") && stdout.contains("byte_index_sum=66"),
        "both slot sums must read 66; stdout:\n{stdout}"
    );
}
