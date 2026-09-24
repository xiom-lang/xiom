// Checker locks for the stdlib-lane relay findings (2026-09-24).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//   m135 -- CLAUSE-position Bool-vs-Int comparison must be REJECTED
//          (it used to compile and coerce silently);
//   m136 -- well-typed clauses (implicit self, @pre, result) stay green.

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

fn run_on(name: &str) -> (String, Option<i32>, PathBuf) {
    let exe = std::env::temp_dir().join(format!(
        "xiom_check_{}_{}{}",
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
fn m135_clause_bool_mix_is_rejected() {
    let (stderr, code, exe) = run_on("m135_clause_bool_int");
    let _ = std::fs::remove_file(&exe);
    assert_ne!(code, Some(0), "clause Bool/Int comparison must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("contract clause compares Bool"),
        "expected the clause Bool-mix diagnostic, got:\n{stderr}"
    );
}

#[test]
fn m136_well_typed_clauses_still_run() {
    let (stderr, code, exe) = run_on("m136_well_typed_clauses");
    assert_eq!(code, Some(0), "well-typed clauses must compile. stderr:\n{stderr}");
    let run = Command::new(&exe).status().expect("run m136");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.code(), Some(0), "m136 must exit 0");
}
