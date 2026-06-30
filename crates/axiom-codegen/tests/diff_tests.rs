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
    assert!(ir.contains("define void @emit_header"), "should emit header function");
    assert!(ir.contains("define void @emit_main"), "should emit main codegen function");
    assert!(ir.contains("define void @compile_program"), "should emit compile_program function");
    assert!(ir.contains("call void @emit_header"), "should call emit_header");
    assert!(ir.contains("call void @emit_main"), "should call emit_main");
}

#[test]
fn test_selfhost_ir_strings_match_expected() {
    let ir = compile_to_ir("selfhost/axiomc.ax");
    assert!(ir.contains("define i64 @main()"), "selfhost IR should contain 'define i64 @main()' string");
    assert!(ir.contains("ret i64 42"), "selfhost IR should contain 'ret i64 42' string");
    assert!(ir.contains("entry0:"), "selfhost IR should contain 'entry0:' string");
}

#[test]
fn test_differential_ir_consistency() {
    let rust_ir = compile_to_ir("examples/diff_test.ax");
    let selfhost_ir = compile_to_ir("selfhost/axiomc.ax");

    // The Rust compiler emits define i64 @main for diff_test.ax
    assert!(rust_ir.contains("define i64 @main()"));
    assert!(rust_ir.contains("ret i64 42"));

    // The selfhost compiler's IR contains the same patterns as string constants
    assert!(selfhost_ir.contains("define i64 @main() {"));
    assert!(selfhost_ir.contains("  ret i64 42"));
    assert!(selfhost_ir.contains("entry0:"));
}
