// m311 (COMPILER_BUGS m283 finding): contract checks whose condition
// constant-folds after inlining must stay enforced on OPTIMIZED builds.
// A violated `requires: x != x` / `ensures: result == x + 999999` on a
// constant input was recorded as pruned by clang -O2 (fires at -O0, gone at
// -O2); compile_contract_check now routes the check's final i1 through a
// volatile no-fold barrier so clang cannot fold the branch at any -O level.
// This lock compiles the fixtures with `--opt-level`:
//   * violated fixtures must abort (exit 1 + `contract violated`) at 0 AND 2;
//   * the satisfied-clause control must stay green at 0 and 2;
//   * the emitted IR must carry the barrier (store/load volatile i8).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m311_o2_contract_barrier")
        .join(name)
}

struct Compiled {
    exe: PathBuf,
    stderr: String,
}

/// Compile `src` with `--opt-level <level>`; panics with the compiler stderr
/// when the compile itself fails (a compile failure is never a test subject
/// here -- the violations under test are runtime-only).
fn compile(src: &Path, level: &str, tag: &str) -> Compiled {
    let dir = std::env::temp_dir().join(format!("xiom_m311_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    let suffix = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let exe = dir.join(format!("m311_{tag}{suffix}"));
    let _ = std::fs::remove_file(&exe);
    let out = Command::new(xiom_bin())
        .arg(src)
        .arg("--opt-level")
        .arg(level)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        out.status.success(),
        "compiling {} at --opt-level {level} must succeed; exit={:?}\nstderr:\n{}",
        src.display(),
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(exe.exists(), "output binary must exist: {}", exe.display());
    Compiled {
        exe,
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

/// Run a compiled fixture; returns (exit code, combined stdout+stderr).
fn run(exe: &Path) -> (Option<i32>, String) {
    let out = Command::new(exe)
        .output()
        .unwrap_or_else(|e| panic!("failed to run '{}': {e}", exe.display()));
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code(), text)
}

/// A violated constant-foldable `requires` must fire at -O0 AND -O2.
#[test]
fn m311_o2_folded_requires_aborts_at_o0_and_o2() {
    let src = fixture("main.xi");
    for level in ["0", "2"] {
        let c = compile(&src, level, &format!("req_o{level}"));
        let (code, text) = run(&c.exe);
        let _ = std::fs::remove_file(&c.exe);
        assert_ne!(
            code,
            Some(0),
            "a violated `requires: x != x` on a constant input must abort at --opt-level {level} \
             (exit code 0 means the check was pruned); output:\n{text}\ncompiler stderr:\n{}",
            c.stderr
        );
        assert!(
            text.contains("contract violated: requires"),
            "expected the requires violation message at --opt-level {level}, got:\n{text}"
        );
    }
}

/// The ensures half: `result == x + 999999` on a constant input must abort
/// at -O2 (fully constant-provable post-inlining).
#[test]
fn m311_o2_folded_ensures_aborts_at_o2() {
    let src = fixture("ens.xi");
    let c = compile(&src, "2", "ens_o2");
    let (code, text) = run(&c.exe);
    let _ = std::fs::remove_file(&c.exe);
    assert_ne!(
        code,
        Some(0),
        "a violated `ensures: result == x + 999999` on a constant input must abort at \
         --opt-level 2; output:\n{text}"
    );
    assert!(
        text.contains("contract violated: ensures"),
        "expected the ensures violation message, got:\n{text}"
    );
}

/// Satisfied constant-foldable clauses must stay green (the barrier must not
/// manufacture failures) at both levels, and the optimized binary must still
/// produce the expected result.
#[test]
fn m311_o2_folded_control_green_at_o0_and_o2() {
    let src = fixture("control.xi");
    for level in ["0", "2"] {
        let c = compile(&src, level, &format!("ctl_o{level}"));
        let (code, text) = run(&c.exe);
        let _ = std::fs::remove_file(&c.exe);
        assert_eq!(
            code,
            Some(0),
            "satisfied constant-foldable clauses must not fail at --opt-level {level}; output:\n{text}"
        );
        assert!(
            text.contains("control ok a=42"),
            "control must complete at --opt-level {level}, got:\n{text}"
        );
    }
}

/// Mechanism lock: the emitted IR must carry the no-fold barrier around the
/// clause branch (store/load volatile i8). Without it, clang -O2 is free to
/// fold the branch condition for constant-provable clauses.
#[test]
fn m311_o2_barrier_in_emitted_ir() {
    let src = fixture("main.xi");
    let out = Command::new(xiom_bin())
        .arg("--emit-ir")
        .arg(&src)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(out.status.success(), "--emit-ir must succeed");
    let ir = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        ir.contains("store volatile i8") && ir.contains("load volatile i8"),
        "the contract no-fold barrier (volatile i8 round-trip) must appear in the emitted IR \
         (m311); IR head:\n{}",
        ir.chars().take(2000).collect::<String>()
    );
}
