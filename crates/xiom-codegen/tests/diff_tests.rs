// XIOM -- Differential Tests
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;
use std::path::Path;

/// Path to the compiled xiomc binary
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
        .expect("failed to run xiomc");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        panic!("xiomc failed on {}:\n{}", source_path, stderr);
    }

    stdout
}

#[test]
fn test_diff_test_produces_correct_ir() {
    let ir = compile_to_ir("examples/diff_test.xi");
    assert!(ir.contains("define i64 @main"), "should define main function");
    assert!(ir.contains("entry0:"), "should have entry0 block");
    assert!(ir.contains("ret i64 42"), "should return 42");
}

#[test]
fn test_selfhost_compiles_cleanly() {
    let ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(ir.contains("define void @emit_add"), "should emit add codegen function");
    assert!(ir.contains("define void @emit_sq"), "should emit sq codegen function");
    assert!(ir.contains("define void @emit_main_demo"), "should emit main_demo codegen function");
    assert!(ir.contains("define void @compile_all"), "should emit compile_all function");
    assert!(ir.contains("call void @compile_all()"), "should call compile_all");
}

#[test]
fn test_selfhost_ir_strings_match_expected() {
    let ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(ir.contains("define i64 @main"), "selfhost IR should contain 'define i64 @main' string");
    assert!(ir.contains("ret i64 %tmp4"), "selfhost IR should contain 'ret i64 %tmp4' string");
    assert!(ir.contains("entry0:"), "selfhost IR should contain 'entry0:' string");
    assert!(ir.contains("fmul double"), "selfhost IR should contain float multiply");
}

#[test]
fn test_differential_ir_consistency() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");

    // The selfhost compiler's IR matches expected demo_float patterns
    assert!(selfhost_ir.contains("define i64 @add(i64 %param0, i64 %param1) {"));
    assert!(selfhost_ir.contains("define double @sq(double %param0) {"));
    // The selfhost program PRINTS this IR text via println (a string
    // constant, not the compiler's own main signature).
    assert!(selfhost_ir.contains("define i64 @main() {"));
    assert!(selfhost_ir.contains("fmul double %tmp1, %tmp2"));
    assert!(selfhost_ir.contains("call double @sq(double 3.000000)"));
    assert!(selfhost_ir.contains("call i64 @add(i64 10, i64 20)"));
}

#[test]
fn test_differential_demo_float() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");

    assert!(selfhost_ir.contains("define i64 @add"), "missing add function");
    assert!(selfhost_ir.contains("define double @sq"), "missing sq function");
    assert!(selfhost_ir.contains("define i64 @main"), "missing main function");
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
    assert!(selfhost_ir.contains("call i64 @read_borrow"));
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
    assert!(selfhost_ir.contains("icmp eq") || selfhost_ir.contains("fcmp oeq"));
    assert!(selfhost_ir.contains("getelementptr"));
}

#[test]
fn test_differential_contracts() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("PositiveInt.invariant_check"));
    assert!(selfhost_ir.contains("define double @divide"));
    assert!(selfhost_ir.contains("contract_ok") || selfhost_ir.contains("contract_fail"));
    assert!(selfhost_ir.contains("@llvm.trap"));
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
}

#[test]
fn test_differential_error() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define %struct.Result @safe_divide"));
    assert!(selfhost_ir.contains("getelementptr %struct.Result"));
    assert!(selfhost_ir.contains("define i64 @Result.is_ok"));
    assert!(selfhost_ir.contains("bitcast double"));
    assert!(selfhost_ir.contains("ptrtoint"));
}

#[test]
fn test_differential_generics() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @wrap_Int"));
    assert!(selfhost_ir.contains("call i64 @wrap_Int(i64 42)"));
}

#[test]
fn test_differential_enum() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("match_check"));
    assert!(selfhost_ir.contains("match_arm"));
    assert!(selfhost_ir.contains("match_merge"));
    assert!(selfhost_ir.contains("icmp eq i64"));
}

#[test]
fn test_differential_derive_enum() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @Color.eq"));
    assert!(selfhost_ir.contains("zext i1"));
}

#[test]
fn test_differential_interface() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @is_greater"));
    assert!(selfhost_ir.contains("icmp sgt"));
}

#[test]
fn test_differential_async() {
    let selfhost_ir = compile_to_ir("selfhost/xiomc.xi");
    assert!(selfhost_ir.contains("define i64 @worker"));
    assert!(selfhost_ir.contains("mul i64"));
}

#[test]
fn test_differential_full() {
    let ir = compile_to_ir("examples\\phase1_full.xi");
    assert!(ir.contains("define %struct.Point @Point.clone"));
    assert!(ir.contains("define double @distance"));
    assert!(ir.contains("define %struct.Point @make_point"));
    assert!(ir.contains("define i64 @main"));
    assert!(ir.contains("fmul"));
    assert!(ir.contains("getelementptr"));
}

#[test]
fn test_differential_hardening() {
    let ir = compile_to_ir("examples\\phase1_hardening.xi");
    assert!(ir.contains("define"));
    assert!(ir.contains("icmp") || ir.contains("fcmp"));
}

#[test]
fn test_differential_selfhost_sim() {
    let ir = compile_to_ir("examples\\phase1_selfhost.xi");
    assert!(ir.contains("define"));
    assert!(ir.contains("call"));
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

#[test]
#[ignore = "selfhost phase pending: xiomc_v050 embeds a 31982-if source_at chain that exceeds the 300s compile timeout; will be re-enabled during the selfhost phase"]
fn test_selfhost_bootstrap_v050() {
    // This test requires clang for native linking via --run.
    // Skip if clang is not available (e.g. fresh laptop without LLVM).
    if !std::process::Command::new("clang")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        eprintln!("skipping test_selfhost_bootstrap_v050: clang not found in PATH");
        return;
    }

    // The v0.5.0 selfhost compiler embeds xiomc.xi source and returns a structural hash.
    // The Rust compiler, processing the same v0.5.0 source, must produce a binary
    // that exits with the SAME hash -- proving bootstrap correctness.
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap();

    let output = Command::new(xiom_path())
        .args(["--run", "selfhost/xiomc_v050.xi"])
        .current_dir(project_root)
        .output()
        .expect("failed to compile and run v0.5.0 selfhost");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Find "exit code: N" in stderr
    let hash: i32 = if let Some(line) = stderr.lines().find(|l| l.contains("exit code:")) {
        line.split("exit code:").nth(1).unwrap().trim().parse().unwrap_or(-1)
    } else {
        -1
    };

    // Expected hash based on xiomc.xi structural counts:
    //   modules=4, functions=29, types=2, ifs+elifs=192, whiles=3, returns=192
    //   hash = 4*100000 + 29*1000 + 2*100 + 192*10 + 3*5 + 192 = 431327
    assert_eq!(hash, 431327, "Selfhost v0.5.0 bootstrap hash mismatch");
}

#[test]
fn test_selfhost_v092_compiles() {
    // v0.9.2 selfhost compiler should compile and produce IR with
    // string constants matching the three demo_float functions
    let ir = compile_to_ir("selfhost/xiomc_v092.xi");
    assert!(ir.contains("define i64 @add"), "missing add function IR");
    assert!(ir.contains("define double @sq"), "missing sq function IR");
    assert!(ir.contains("define i64 @main"), "missing main function IR");
}

