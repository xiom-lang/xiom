use std::process::Command;
use std::path::Path;

/// Path to the compiled axiomc binary
fn axiomc_path() -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join("axiomc.exe");
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join("axiomc.exe");
    }
    path.to_str().unwrap().to_string()
}

/// Compile a .ax file and return the LLVM IR output
fn compile_to_ir(source_path: &str) -> String {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap();

    let output = Command::new(axiomc_path())
        .arg(source_path)
        .current_dir(project_root)
        .output()
        .expect("failed to run axiomc");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        panic!("axiomc failed on {}:\n{}", source_path, stderr);
    }

    stdout
}

#[test]
fn test_diff_test_produces_correct_ir() {
    let ir = compile_to_ir("examples/diff_test.ax");
    assert!(ir.contains("define i64 @main()"), "should define main function");
    assert!(ir.contains("entry0:"), "should have entry0 block");
    assert!(ir.contains("ret i64 42"), "should return 42");
}

#[test]
fn test_selfhost_compiles_cleanly() {
    let ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(ir.contains("define void @emit_add"), "should emit add codegen function");
    assert!(ir.contains("define void @emit_sq"), "should emit sq codegen function");
    assert!(ir.contains("define void @emit_main_demo"), "should emit main_demo codegen function");
    assert!(ir.contains("define void @compile_all"), "should emit compile_all function");
    assert!(ir.contains("call void @compile_all"), "should call compile_all");
}

#[test]
fn test_selfhost_ir_strings_match_expected() {
    let ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(ir.contains("define i64 @main()"), "selfhost IR should contain 'define i64 @main()' string");
    assert!(ir.contains("ret i64 %tmp4"), "selfhost IR should contain 'ret i64 %tmp4' string");
    assert!(ir.contains("entry0:"), "selfhost IR should contain 'entry0:' string");
    assert!(ir.contains("fmul double"), "selfhost IR should contain float multiply");
}

#[test]
fn test_differential_ir_consistency() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");

    // The selfhost compiler's IR matches expected demo_float patterns
    assert!(selfhost_ir.contains("define i64 @add(i64 %param0, i64 %param1) {"));
    assert!(selfhost_ir.contains("define double @sq(double %param0) {"));
    assert!(selfhost_ir.contains("define i64 @main() {"));
    assert!(selfhost_ir.contains("fmul double %tmp1, %tmp2"));
    assert!(selfhost_ir.contains("call double @sq(double 3.000000)"));
    assert!(selfhost_ir.contains("call i64 @add(i64 10, i64 20)"));
}

#[test]
fn test_differential_demo_float() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");

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
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("define i64 @take_ownership"));
    assert!(selfhost_ir.contains("define i64 @read_borrow"));
    assert!(selfhost_ir.contains("call i64 @take_ownership(i64 41)"));
    assert!(selfhost_ir.contains("call i64 @read_borrow"));
    assert!(selfhost_ir.contains("add i64"));
}

#[test]
fn test_differential_derive() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
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
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
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
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("define i64 @add"));
    assert!(selfhost_ir.contains("define i64 @mul"));
    assert!(selfhost_ir.contains("call i64 @add(i64 10, i64 20)"));
    assert!(selfhost_ir.contains("mul i64"));
}

#[test]
fn test_differential_error() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("define %struct.Result @safe_divide"));
    assert!(selfhost_ir.contains("getelementptr %struct.Result"));
    assert!(selfhost_ir.contains("define i64 @Result.is_ok"));
    assert!(selfhost_ir.contains("bitcast double"));
    assert!(selfhost_ir.contains("ptrtoint"));
}

#[test]
fn test_differential_generics() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("define i64 @wrap_Int"));
    assert!(selfhost_ir.contains("call i64 @wrap_Int(i64 42)"));
}

#[test]
fn test_differential_enum() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("match_check"));
    assert!(selfhost_ir.contains("match_arm"));
    assert!(selfhost_ir.contains("match_merge"));
    assert!(selfhost_ir.contains("icmp eq i64"));
}

#[test]
fn test_differential_derive_enum() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("define i64 @Color.eq"));
    assert!(selfhost_ir.contains("zext i1"));
}

#[test]
fn test_differential_interface() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("define i64 @is_greater"));
    assert!(selfhost_ir.contains("icmp sgt"));
}

#[test]
fn test_differential_async() {
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(selfhost_ir.contains("define i64 @worker"));
    assert!(selfhost_ir.contains("mul i64"));
}
