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
    compile_with_flags(name, &[])
}

/// Compile with extra CLI flags (e.g. `--strict`).
fn compile_with_flags(name: &str, extra_args: &[&str]) -> (String, Option<i32>, PathBuf) {
    let exe = std::env::temp_dir().join(format!(
        "xiom_e001_{}_{}{}",
        name,
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let mut cmd = Command::new(xiom_bin());
    cmd.arg(fixture(name)).args(extra_args).arg("-o").arg(&exe);
    let output = cmd
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

// m318 (REL-20261013-2215-engine-2): PIN the by-value move semantic. The
// borrow checker treats non-Copy types (structs/Vecs/enums/tuples) as
// move-only: a bare by-value argument consumes the LOCAL, so reuse trips
// E001 "use of moved value" -- a warning by default (compilation continues;
// the ABI still passes a copy, so the runtime value stays readable) and a
// hard abort under --strict. Encodings of the four faces live in the fixture
// (local struct reuse = expected E001; scalar reuse, `&` reuse, and by-value
// param forwarding = NO E001 -- the engine's by-ref-reuse convention). NOTE:
// the checker currently prints the message TWICE for one reuse site
// (check_use + move_var both report); the assertions below deliberately do
// not freeze that duplicate count.
#[test]
fn m318_by_value_struct_move_semantics_pinned() {
    let (stderr, code, exe) = compile("m318_move_by_value_pin");
    assert_eq!(code, Some(0), "default mode must compile (E001 = warning). stderr:\n{stderr}");
    assert!(
        stderr.contains("use of moved value 's'"),
        "local struct reuse by value must warn 'use of moved value':\n{stderr}"
    );
    // Every E001 diagnostic must be the local-struct reuse of `s` -- the
    // scalar, by-ref and param faces must stay clean.
    for line in stderr.lines().filter(|l| l.contains("E001")) {
        assert!(
            line.contains("use of moved value 's'"),
            "unexpected E001 diagnostic on a Copy/by-ref/param face: {line}"
        );
    }
    let run = Command::new(&exe).status().expect("run m318 move pin");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.code(), Some(0), "default-mode program must exit 0 (values pinned in the fixture)");

    // Strict mode: the pinned semantic becomes a hard error.
    let (strict_stderr, strict_code, _exe) = compile_with_flags("m318_move_by_value_pin", &["--strict"]);
    assert_ne!(strict_code, Some(0), "--strict must abort on the reuse. stderr:\n{strict_stderr}");
    assert!(
        strict_stderr.contains("use of moved value 's'"),
        "--strict must report the moved-value error:\n{strict_stderr}"
    );
    assert!(
        strict_stderr.contains("aborting compilation"),
        "--strict must abort compilation:\n{strict_stderr}"
    );
}
