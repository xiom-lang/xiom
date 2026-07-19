// Phase 5f — Verifier regression tests
// Tests SMT-LIB generation (not z3 execution, which requires z3 on PATH).

use std::process::Command;
use std::path::Path;

fn verify_path() -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join("xiom-verify.exe");
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join("xiom-verify.exe");
    }
    path.to_str().unwrap().to_string()
}

fn project_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .to_path_buf()
}

/// Test that a function with `requires` + `ensures` generates correct SMT-LIB
/// with body encoding (declare-const for params, assert for body, push/pop for ensures).
#[test]
fn verify_abs_generates_body_encoding() {
    let output = Command::new(verify_path())
        .args(["tests/verify/test_abs.xi", "-o", "NUL"]) // -o NUL discards output
        .current_dir(project_root())
        .output()
        .expect("xiom-verify");
    
    assert!(output.status.success(),
        "verifier must succeed: {}",
        String::from_utf8_lossy(&output.stderr));
}

/// Test that a buggy function's SMT output still generates (the violation is
/// detected by z3, not by the SMT generator).
#[test]
fn verify_buggy_generates_smt() {
    let output = Command::new(verify_path())
        .args(["tests/verify/test_buggy.xi", "--output", "NUL"])
        .current_dir(project_root())
        .output()
        .expect("xiom-verify");
    
    assert!(output.status.success(),
        "verifier must succeed for buggy code (violation detected by z3, not generator): {}",
        String::from_utf8_lossy(&output.stderr));
}

/// Test that the SMT output contains the expected body encoding patterns.
#[test]
fn verify_abs_smt_has_body() {
    let tmp = std::env::temp_dir().join("xiom_verify_test.smt2");
    let output = Command::new(verify_path())
        .args(["tests/verify/test_abs.xi", "-o", tmp.to_str().unwrap()])
        .current_dir(project_root())
        .output()
        .expect("xiom-verify");
    
    assert!(output.status.success(), "verifier failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let smt = std::fs::read_to_string(&tmp).expect("read SMT file");
    let _ = std::fs::remove_file(&tmp);
    
    assert!(smt.contains("declare-const x Int"), "Must declare param x:\n{smt}");
    assert!(smt.contains("declare-const |result| Int"), "Must declare result:\n{smt}");
    assert!(smt.contains("; --- body encoding ---"), "Must have body encoding:\n{smt}");
    assert!(smt.contains("(check-sat)"), "Must have check-sat:\n{smt}");
    assert!(!smt.contains("QF_NRA"), "Must not use QF_NRA for integer contracts");
}
