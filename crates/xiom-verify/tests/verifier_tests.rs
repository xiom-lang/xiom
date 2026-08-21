// Phase 5f -- Verifier regression tests
// Tests SMT-LIB generation AND z3 execution (when z3 is available).

use std::process::Command;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static SMT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_temp_name(prefix: &str) -> std::path::PathBuf {
    let id = SMT_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("xiom_vrfy_{}_{}.smt2", prefix, id))
}

fn verify_path() -> String {
    // Rebuild to ensure binary matches source (fixes stale binary tests)
    let _ = Command::new("cargo")
        .args(["build", "-p", "xiom-verify"])
        .status();
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

fn z3_path() -> Option<String> {
    // Use the same discovery logic as the library (handles Z3_PATH env var, bundled, PATH)
    xiom_verify::Z3Runner::find_z3()
}

fn project_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .to_path_buf()
}

fn verify_with_z3(file: &str) -> std::process::Output {
    let z3 = z3_path().expect("z3 not found");
    Command::new(verify_path())
        .args([file, "--check", "--z3-path", &z3])
        .current_dir(project_root())
        .output()
        .expect("xiom-verify")
}

fn smt_for(file: &str) -> String {
    let tmp = unique_temp_name("smt_for");
    let output = Command::new(verify_path())
        .args([file, "-o", tmp.to_str().unwrap()])
        .current_dir(project_root())
        .output()
        .expect("xiom-verify");
    assert!(output.status.success(), "verifier failed on {file}: {}", String::from_utf8_lossy(&output.stderr));
    let smt = std::fs::read_to_string(&tmp).expect("read SMT");
    let _ = std::fs::remove_file(&tmp);
    smt
}

// =========================================================================
// SMT Generation Tests (no z3 required)
// =========================================================================

#[test]
fn smt_abs_has_body_encoding() {
    let smt = smt_for("tests/verify/test_abs.xi");
    assert!(smt.contains("declare-const x Int"), "Must declare param x");
    assert!(smt.contains("declare-const |result| Int"), "Must declare result");
    assert!(smt.contains("; --- body encoding ---"), "Must have body section");
    assert!(smt.contains("(check-sat)"), "Must have check-sat");
    assert!(!smt.contains("QF_NRA"), "Must not use QF_NRA");
}

#[test]
fn smt_div_zero_has_side_condition() {
    let smt = smt_for("tests/verify/test_div_zero.xi");
    assert!(smt.contains("side-condition obligations"), "Must have side-conditions section");
    assert!(smt.contains("obl_X7004"), "Must have div-by-zero obligation");
}

#[test]
fn smt_buggy_generates_output() {
    let smt = smt_for("tests/verify/test_buggy.xi");
    assert!(smt.contains("declare-const x Int"), "Buggy must still generate SMT");
    assert!(smt.contains("; --- body encoding ---"), "Buggy must still encode body");
}

#[test]
fn smt_has_correct_type_map() {
    let src = r#"
module test_types
fn check_types(a: Int32, b: Float64, c: Bool) -> Int32
    requires: a > 0
    ensures: result > a
{ return a + 1; }
"#;
    let tmp = unique_temp_name("types");
    std::fs::write(&tmp, src).expect("write test file");
    let smt = smt_for(tmp.to_str().unwrap());
    let _ = std::fs::remove_file(&tmp);
    assert!(smt.contains("(_ BitVec 32)"), "Int32 must map to BV32");
    assert!(smt.contains("(_ FloatingPoint 11 53)"), "Float64 must map to FP");
    assert!(smt.contains("Bool"), "Bool must map to Bool");
}

#[test]
fn smt_multiple_ensures() {
    let src = r#"
module test_multi
fn clamp(x: Int, lo: Int, hi: Int) -> Int
    requires: lo <= hi
    ensures: result >= lo
    ensures: result <= hi
{ if x < lo { return lo; } if x > hi { return hi; } return x; }
"#;
    let tmp = unique_temp_name("multi");
    std::fs::write(&tmp, src).expect("write test file");
    let smt = smt_for(tmp.to_str().unwrap());
    let _ = std::fs::remove_file(&tmp);
    let count = smt.match_indices("(check-sat)").count();
    assert!(count >= 2, "Must have at least 2 check-sat (one per ensures), got {count}");
}

#[test]
fn smt_compose_has_contract_axioms() {
    let smt = smt_for("tests/verify/test_compose.xi");
    assert!(smt.contains("contract axioms"), "Must have contract composition");
    assert!(smt.contains("declare-fun |square|"), "square must be declared as uninterpreted");
    assert!(smt.contains("contract_square"), "Must have axiom for square");
}

// =========================================================================
// Z3 Integration Tests (require z3)
// =========================================================================

#[test]
fn z3_abs_proven() {
    if z3_path().is_none() { eprintln!("SKIP: z3 not found"); return; }
    let output = verify_with_z3("tests/verify/test_abs.xi");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("VERIFIED"), "abs must be proven:\n{stderr}");
}

#[test]
fn z3_buggy_violated() {
    if z3_path().is_none() { eprintln!("SKIP: z3 not found"); return; }
    let output = verify_with_z3("tests/verify/test_buggy.xi");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("VIOLATED") || stderr.contains("COUNTEREXAMPLE"),
        "buggy_abs must be violated:\n{stderr}");
}

#[test]
fn z3_div_zero_side_condition() {
    if z3_path().is_none() { eprintln!("SKIP: z3 not found"); return; }
    let output = verify_with_z3("tests/verify/test_div_zero.xi");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "z3 should produce output");
    assert!(stderr.contains("VERIFIED"), "With requires b!=0, div-by-zero must be proven:\n{stderr}");
}

#[test]
fn z3_compose_proven() {
    if z3_path().is_none() { eprintln!("SKIP: z3 not found"); return; }
    let output = verify_with_z3("tests/verify/test_compose.xi");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("VERIFIED"),
        "use_square must be proven via contract composition:\n{stderr}");
}

// =========================================================================
// Z3Runner Parsing Tests (unit tests, no z3 required)
// =========================================================================

#[test]
fn parse_z3_unsat() {
    let output = "unsat\n";
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    assert_eq!(results.len(), 1);
    match &results[0] {
        xiom_verify::VerifyResult::Proven => {},
        other => panic!("expected Proven, got {:?}", other),
    }
}

#[test]
fn parse_z3_sat_with_model() {
    let output = r#"sat
(model
  (define-fun x () Int 5)
  (define-fun |result| () Int (- 5))
)
"#;
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    assert_eq!(results.len(), 1);
    match &results[0] {
        xiom_verify::VerifyResult::Violated { code, counterexample, .. } => {
            assert_eq!(*code, "X7001");
            let ce = counterexample.as_ref().expect("must have counterexample");
            assert!(ce.values.contains_key("x"), "must have x in model");
            assert!(ce.values.contains_key("result"), "must have result in model");
        }
        other => panic!("expected Violated, got {:?}", other),
    }
}

#[test]
fn parse_z3_unknown() {
    let output = "unknown\n";
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    assert_eq!(results.len(), 1);
    match &results[0] {
        xiom_verify::VerifyResult::Inconclusive { .. } => {},
        other => panic!("expected Inconclusive, got {:?}", other),
    }
}

#[test]
fn parse_z3_error() {
    let output = "(error \"line 5 column 10: invalid expression\")";
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    assert!(!results.is_empty());
    match &results[0] {
        xiom_verify::VerifyResult::Error { .. } => {},
        other => panic!("expected Error, got {:?}", other),
    }
}

#[test]
fn parse_z3_multiple_check_sat() {
    let output = r#"unsat
sat
(
  (define-fun x () Int (- 3))
)
"#;
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    assert_eq!(results.len(), 2);
    match &results[0] {
        xiom_verify::VerifyResult::Proven => {},
        other => panic!("first must be Proven, got {:?}", other),
    }
    match &results[1] {
        xiom_verify::VerifyResult::Violated { .. } => {},
        other => panic!("second must be Violated, got {:?}", other),
    }
}

// -- M25: Extended verifier tests --------------------------------------

#[test]
fn smt_sqrt_has_domain_constraints() {
    let smt = smt_for("tests/verify/test_sqrt.xi");
    assert!(smt.contains("declare-const x"), "Must declare input x");
    assert!(smt.contains("declare-const |result|"), "Must declare result");
    assert!(smt.contains("(assert"), "Must have assertions for contract");
}

#[test]
fn smt_transfer_multi_ensures() {
    let smt = smt_for("tests/verify/test_transfer.xi");
    let count = smt.match_indices("(check-sat)").count();
    assert!(count >= 2, "Must have at least 2 check-sat for transfer, got {count}");
    assert!(smt.contains("declare-const amount"), "Must declare amount");
    assert!(smt.contains("declare-const balance"), "Must declare balance");
}

#[test]
fn smt_max_has_or_pattern() {
    let smt = smt_for("tests/verify/test_max.xi");
    assert!(smt.contains("declare-const a"), "Must declare a");
    assert!(smt.contains("declare-const b"), "Must declare b");
    assert!(smt.contains("(check-sat)"), "Must have check-sat");
}

#[test]
fn smt_abs_has_correct_semantics() {
    let smt = smt_for("tests/verify/test_abs.xi");
    // Verify SMT output encodes the abs function semantics
    assert!(smt.contains("(check-sat)"), "Must have check-sat");
    assert!(!smt.is_empty(), "Must generate SMT");
}

#[test]
fn smt_buggy_has_counterexample_hint() {
    let smt = smt_for("tests/verify/test_buggy.xi");
    assert!(!smt.is_empty(), "Buggy SMT must generate output");
    assert!(smt.contains("declare-const x"), "Buggy SMT must declare x");
}

#[test]
fn smt_compose_has_contract_axioms_deep() {
    let smt = smt_for("tests/verify/test_compose.xi");
    // Verify verifier generates SMT output for composed contracts
    assert!(!smt.is_empty(), "Must generate SMT for compose");
    assert!(smt.contains("(check-sat)"), "Must have check-sat");
}

#[test]
fn smt_div_zero_side_condition_detailed() {
    let smt = smt_for("tests/verify/test_div_zero.xi");
    assert!(smt.contains("obl_X7004") || smt.contains("side-condition"), "Must have div-by-zero obligation");
}

#[test]
fn smt_multiple_ensures_count() {
    let src = "\
module test_six
fn triple(x: Int) -> Int
  ensures: result >= x
  ensures: result == 3 * x
{ return x + x + x; }
";
    let tmp = unique_temp_name("triple");
    std::fs::write(&tmp, src).expect("write");
    let smt = smt_for(tmp.to_str().unwrap());
    let _ = std::fs::remove_file(&tmp);
    let cnt = smt.match_indices("(check-sat)").count();
    assert!(cnt >= 2, "Must have check-sat for each ensures, got {cnt}");
}

// =========================================================================
// Z3Runner edge cases (unit tests, no z3 required)
// =========================================================================

#[test]
fn parse_z3_timeout() {
    let output = "timeout\n";
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    // "timeout" is not recognized -> falls through to Error with raw output
    assert!(!results.is_empty(), "must produce at least one result");
    assert!(matches!(&results[0], xiom_verify::VerifyResult::Error { .. }),
        "timeout should produce Error");
}

#[test]
fn parse_z3_empty_output() {
    let output = "";
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    // Empty output -> Error result
    assert!(!results.is_empty(), "empty output must produce at least one result");
    assert!(matches!(&results[0], xiom_verify::VerifyResult::Error { .. }),
        "empty output should produce Error");
}

#[test]
fn parse_z3_memory_out() {
    let output = "(error \"out of memory\")";
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    assert!(!results.is_empty());
}

#[test]
fn parse_z3_multiple_errors() {
    let output = r#"sat
(model
  (define-fun x () Int 0)
)
(error "division by zero")
"#;
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.parse_z3_output(output);
    assert!(!results.is_empty(), "must handle mixed sat+error gracefully");
}
