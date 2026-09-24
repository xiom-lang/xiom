// E001 conservatism locks (Sprint C). Copyright (c) 2026 Eleftherios Notas
// and The XIOM Authors. SPDX-License-Identifier: MIT OR Apache-2.0
//
// Runs the built compiler on two fixtures:
//   m132 -- temporary borrows consumed by a statement must NOT warn;
//   m133 -- a still-live immutable borrow followed by `&mut` MUST warn.
// The genuine-overlap warning is intentionally part of the contract, so this
// file asserts on stderr rather than exit codes alone.

use std::path::PathBuf;
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join(name)
        .join("main.xi")
}

/// Compile a fixture to a temp output; returns (stderr, exit code, exe path).
fn compile(name: &str) -> (String, Option<i32>, PathBuf) {
    let exe = std::env::temp_dir().join(format!(
        "xiom_e001_{}_{}{}",
        name,
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg(fixture(name))
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
        exe,
    )
}

#[test]
fn m132_consumed_temporary_borrows_do_not_warn_e001() {
    let (stderr, code, exe) = compile("m132_e001_consumed_borrow");
    assert_eq!(code, Some(0), "compile failed. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[E001]"),
        "false-positive E001 diagnostic:\n{stderr}"
    );
    let run = Command::new(&exe).status().expect("run m132");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.code(), Some(0), "m132 must exit 0");
}

#[test]
fn m133_genuine_overlap_still_warns_e001() {
    let (stderr, code, exe) = compile("m133_e001_genuine_overlap");
    assert_eq!(code, Some(0), "compile failed. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[E001]"),
        "the genuine-overlap E001 warning must survive the conservatism fix:\n{stderr}"
    );
    let run = Command::new(&exe).status().expect("run m133");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.code(), Some(0), "m133 must exit 0");
}
