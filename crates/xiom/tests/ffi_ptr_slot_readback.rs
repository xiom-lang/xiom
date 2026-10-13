// m281 (REL-20261012-2051-bindings-2 B-24 + REL-20261010-1549-bindings B-20,
// 2026-10-13): C-written slot readback through raw-pointer indexing. Pre-fix
// `p[i]` on a `*UInt8` param lowered to `xiom_char_at` (a UTF-8 CODEPOINT
// decoder clamped by strnlen) instead of a raw byte load, so a binary buffer
// whose byte 0 is 0x00 read back all zeros (little-endian 1024/1280 slots;
// foreign mesh/float buffers read zeros while C saw the data). The fixture
// writes through the owned slot from C, then reads it back through a local
// pointer binding, the direct `as_mut_ptr()[i]` form, and ffi.ptr_read_*.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

#[test]
fn m281_ffi_ptr_slot_readback() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m281_ffi_ptr_slot_readback");
    let dir = std::env::temp_dir().join(format!("xiom_m281_ptr_{}", std::process::id()));
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
        "m281 ptr-readback fixture must compile; exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&out_bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", out_bin.display()));
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "C-written slot must read back through all pointer forms; exit={:?}\nstdout:\n{stdout}\nstderr:\n{}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        stdout.contains("m281 ptr slot readback ok"),
        "probe marker missing; stdout:\n{stdout}"
    );
}
