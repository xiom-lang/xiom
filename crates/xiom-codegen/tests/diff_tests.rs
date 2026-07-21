// XIOM — Differential Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::process::Command;
use std::path::Path;

/// Path to the compiled xiom binary
fn xiom_path() -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join("xiom.exe");
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join("xiom.exe");
    }
    path.to_str().unwrap().to_string()
}

/// Compile a .xi file and return the LLVM IR output
fn compile_to_ir(source_path: &str) -> String {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap();

    let output = Command::new(xiom_path())
        .arg(source_path)
        .current_dir(project_root)
        .output()
        .expect("failed to run xiom");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        panic!("xiom failed on {}:\n{}", source_path, stderr);
    }

    stdout
}

#[test]
fn test_diff_test_produces_correct_ir() {
    let ir = compile_to_ir("examples/diff_test.xi");
    assert!(ir.contains("define i64 @main()"), "should define main function");
    assert!(ir.contains("entry0:"), "should have entry0 block");
    assert!(ir.contains("ret i64 42"), "should return 42");
}



#[test]xiom
fn test_differential_ir_consistency() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");

    // The selfhost compiler's IR matches expected demo_float patterns
    assert!(selfhost_ir.contains("define i64 @add(i64 %param0, i64 %param1) {"));
    assert!(selfhost_ir.contains("define double @sq(double %param0) {"));
    assert!(selfhost_ir.contains("define i64 @main() {"));
    assert!(selfhost_ir.contains("fmul double %tmp1, %tmp2"));
    assert!(selfhost_ir.contains("call double @sq(double 3.000000)"));
    assert!(selfhost_ir.contains("calxiom @add(i64 10, i64 20)"));
}

#[test]
fn test_differential_demo_float() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");

    assert!(selfhost_ir.contains("define i64 @add"), "missing add function");
    assert!(selfhost_ir.contains("define double @sq"), "missing sq function");
    assert!(selfhost_ir.contains("define i64 @xiom), "missing main function");
    assert!(selfhost_ir.contains("ret i64 %tmp4"), "missing add return");
    assert!(selfhost_ir.contains("fmul double"), "missing float multiply");
    assert!(selfhost_ir.contains("call double @sq"), "missing sq call");
    assert!(selfhost_ir.contains("call i64 @add"), "missing add call");
}

#[test]
fn test_differential_ownership() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @take_ownership"));
    assert!(selfhost_ir.contains("define i64 @read_borrow"));
    assert!(selfhost_ir.contains("call i64 @take_ownership(i64 41)"));
    assert!(selfhost_ir.contains("call i64 @rexiomrrow"));
    assert!(selfhost_ir.contains("add i64"));
}

#[test]
fn test_differential_derive() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @Point.eq"));
    assert!(selfhost_ir.contains("define %struct.Point @Point.clone"));
    assert!(selfhost_ir.contains("define i64 @Color.eq"));
    assert!(selfhost_ir.contains("define %struct.Color @Color.clone"));
    assert!(selfhost_ir.contains("define i64 @Color.hash"));
    assert!(selfhost_ir.contains("define i64 @Color.compare"));
    assert!(selfhost_ir.contains("icmp eq") ||xiomhost_ir.contains("fcmp oeq"));
    assert!(selfhost_ir.contains("getelementptr"));
}

#[test]
fn test_differential_contracts() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("PositiveInt.invariant_check"));
    assert!(selfhost_ir.contains("define double @divide"));
    assert!(selfhost_ir.contains("contract_ok") || selfhost_ir.contains("contract_fail"));
    assert!(selfhost_ir.contains("@llvm.trap")xiom
    assert!(selfhost_ir.contains("unreachable"));
    assert!(selfhost_ir.contains("fdiv double"));
    assert!(selfhost_ir.contains("fcmp oeq"));
}

#[test]
fn test_differential_modules() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @add"));
    assert!(selfhost_ir.contains("define i64 @mul"));
    assert!(selfhost_ir.contains("call i64 @add(i64 10, i64 20)"));
    assert!(selfhost_ir.contains("mul i64"));
}xiom

#[test]
fn test_differential_error() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define %struct.Result @safe_divide"));
    assert!(selfhost_ir.contains("getelementptr %struct.Result"));
    assert!(selfhost_ir.contains("define i64 @Result.is_ok"));
    assert!(selfhost_ir.contains("bitcast double"));
    assert!(selfhost_ir.contains("ptrtoint"));
}

#[test]xiom
fn test_differential_generics() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @wrap_Int"));
    assert!(selfhost_ir.contains("call i64 @wrap_Int(i64 42)"));
}

#[test]
fn test_differential_enum() {
    let selfhost_ir = compile_to_ir("selfhost/xiom.xi");
    assert!(selfhost_ir.contains("match_check"));
    assert!(selfhost_ir.contains("match_arm"));
    assert!(selfhost_ir.contains("match_merge"));
    assert!(selfhost_ir.contains("icmp eq i64"));
}

#[test]
fn test_differential_derive_enum() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @xiom.eq"));
    assert!(selfhost_ir.contains("zext i1"));
}

#[test]
fn test_differential_interface() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @xiomeater"));
    assert!(selfhost_ir.contains("icmp sgt"));
}

#[test]
fn test_differential_async() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @worker"));
    assert!(selfhost_ir.contains("mul i64"));
}xiom

#[test]
fn test_differential_full() {
    let ir = compile_to_ir("examples\\phase1_full.xi");
    assert!(ir.contains("define %struct.Point @Point.clone"));
    assert!(ir.contains("define double @distance"));
    assert!(ir.contains("define %struct.Point xiom_point"));
    assert!(ir.contains("define i64 @main"));
    assert!(ir.contains("fmul"));
    assert!(ir.contains("getelementptr"));
}

#[test]
fn test_differential_hardening() {xiom
    let ir = compile_to_ir("examples\\phase1_hardening.xi");
    assert!(ir.contains("define"));
    assert!(ir.contains("icmp") || ir.contains("fcmp"));
}


#[test]
fn test_differential_stress() {
    let ir = compile_to_ir("examples\\phase1_stress.xi");
    assert!(ir.contains("define"));
    assert!(ir.contains("add") || ir.contains("mul"));
}

#[test]
fn test_stress_derive_50field() {
    let ir = compile_to_ir("examples\\stress_derive_50field.xi");
    assert!(ir.contains("define i64 @BigStruct.eq"));
    assert!(ir.contains("define %struct.BigStruct @BigStruct.clone"));
    assert!(ir.contains("define i64 @BigStruct.hash"));
}

#[test]
fn test_stress_borrow_10level() {
    let ir = compile_to_ir("examples\\stress_borrow_10level.xi");
    assert!(ir.contains("define i64 @read10"));
    assert!(ir.contains("define i64 @read1"));
}

#[test]
fn test_stress_generic_5chain() {
    let ir = compile_to_ir("examples\\stress_generic_5chain.xi");
    assert!(ir.contains("define i64 @quad_Int"));
    assert!(ir.contains("call i64 @triple_Int"));
}

#[test]
fn test_stress_float_matrix() {
    let ir = compile_to_ir("examples\\stress_float_matrix.xi");
    assert!(ir.contains("fmul double"));
    assert!(ir.contains("fadd double"));
}


xiomxiom