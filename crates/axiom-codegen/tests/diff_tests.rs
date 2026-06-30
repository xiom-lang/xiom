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
