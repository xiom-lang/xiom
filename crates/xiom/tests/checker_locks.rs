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
fn m141_mixed_brackets_lax_then_strict() {
    // LAX transition default: the legacy mixed spelling still compiles+run.
    let (stderr, code, exe) = run_on("m141_mixed_bracket_switch");
    assert_eq!(code, Some(0), "lax default must accept the mixed spelling. stderr:\n{stderr}");
    let run = Command::new(&exe).status().expect("run m141");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.code(), Some(0), "m141 must exit 0");

    // STRICT switch: the closer must match the opener -> P001.
    let exe = std::env::temp_dir().join(format!(
        "xiom_brackets_strict_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg(fixture("m141_mixed_bracket_switch"))
        .arg("-o")
        .arg(&exe)
        .env("XIOM_STRICT_BRACKETS", "1")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let _ = std::fs::remove_file(&exe);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "strict mode must reject. stderr:\n{stderr}");
    assert!(
        stderr.contains("expected '>'") || stderr.contains("expected ']'"),
        "expected a bracket P001 diagnostic, got:\n{stderr}"
    );
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

#[test]
fn m137_strict_clause_mode_rejects_non_bool_predicates() {
    // Default (light) mode: `requires: x` (Int) is accepted.
    let (stderr, code, exe) = run_on("m137_clause_non_bool");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "light mode must accept the fixture. stderr:\n{stderr}");

    // Strict mode: the full predicate rule rejects it. The stdlib lane runs
    // the same switch over the catalog corpus before the default flips.
    let exe = std::env::temp_dir().join(format!(
        "xiom_check_strict_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg(fixture("m137_clause_non_bool"))
        .arg("-o")
        .arg(&exe)
        .env("XIOM_STRICT_CLAUSES", "1")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let _ = std::fs::remove_file(&exe);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "strict mode must reject. stderr:\n{stderr}");
    assert!(
        stderr.contains("contract clause must be Bool"),
        "expected the strict predicate diagnostic, got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Stage 6 lint wave (W002/W003): warning-only, user-program scope. Each lint
// gets a positive fixture (code in stderr AND compile exit 0) and a negative
// fixture that must stay silent.
// ---------------------------------------------------------------------------

#[test]
fn m151_w002_unconditional_cycle_warns() {
    let (stderr, code, exe) = run_on("m151_w002_cycle");
    let _ = std::fs::remove_file(&exe);
    // Compile exit 0: the lint is warning-only and must never block a build.
    // (The fixture is deliberately NOT executed -- the cycle cannot
    // terminate.)
    assert_eq!(code, Some(0), "W002 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W002]"),
        "expected warning[W002] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unconditional recursive cycle"),
        "expected the cycle message, got:\n{stderr}"
    );
}

#[test]
fn m151_w002_guard_stays_silent() {
    let (stderr, code, exe) = run_on("m151_w002_guard");
    assert_eq!(code, Some(0), "guard-first recursion must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W002]"),
        "guard-first recursion must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m151_w002_guard");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the guard fixture must run cleanly");
}

#[test]
fn m151_w003_unreachable_warns() {
    let (stderr, code, exe) = run_on("m151_w003_unreachable");
    assert_eq!(code, Some(0), "W003 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W003]"),
        "expected warning[W003] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unreachable statement"),
        "expected the unreachable-statement message, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m151_w003_unreachable");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the W003 fixture must still run cleanly");
}

#[test]
fn m151_w003_guarded_stays_silent() {
    let (stderr, code, exe) = run_on("m151_w003_guarded");
    assert_eq!(code, Some(0), "guarded fixture must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W003]"),
        "guarded shapes must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m151_w003_guarded");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the guarded fixture must run cleanly");
}
