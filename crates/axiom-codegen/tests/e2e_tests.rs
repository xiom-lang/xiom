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

fn project_root() -> &'static Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .to_path_buf()
    }).as_path()
}

/// Compile an AXIOM source file to a native binary and return the exit code
fn compile_and_run(source_path: &str) -> Option<i32> {
    let source = Path::new(source_path);
    let exe_name = format!("e2e_{}.exe", source.file_stem()?.to_str()?);

    let bin_path = axiomc_path();

    // Compile
    let compile = Command::new(&bin_path)
        .args(["-o", &exe_name, source_path])
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{bin_path}': {e}"));

    if !compile.status.success() {
        let stderr = String::from_utf8_lossy(&compile.stderr);
        let stdout = String::from_utf8_lossy(&compile.stdout);
        eprintln!("compile failed for {source_path}");
        eprintln!("stdout: {stdout}");
        eprintln!("stderr: {stderr}");
        return None;
    }

    // Run
    let exe_path = project_root().join(&exe_name);
    let run = Command::new(&exe_path)
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{:?}': {e}", exe_path));

    run.status.code()
}

/// Compile to WASM and verify file exists
fn compile_wasm(source_path: &str) -> bool {
    let source = Path::new(source_path);
    let wasm_name = format!("e2e_{}.wasm", source.file_stem().unwrap().to_str().unwrap());
    let bin_path = axiomc_path();
    let output = Command::new(&bin_path)
        .args(["--target", "wasm", "-o", &wasm_name, source_path])
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{bin_path}': {e}"));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("WASM compile failed for {source_path}");
        eprintln!("stdout: {stdout}");
        eprintln!("stderr: {stderr}");
    }
    output.status.success() && project_root().join(&wasm_name).exists()
}

/// Compile to IR and verify output contains expected text
fn compile_and_check_ir(source_path: &str, expected_ir: &str) -> bool {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed to emit IR");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(expected_ir)
}

/// Compile with --diagnostics=json and verify valid JSON
fn compile_diagnostics_json(source_path: &str) -> bool {
    let output = Command::new(axiomc_path())
        .args(["--diagnostics=json", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("\"status\"")
}

/// Compile with --dump-contracts and verify output
fn compile_dump_contracts(source_path: &str) -> bool {
    let output = Command::new(axiomc_path())
        .args(["--dump-contracts", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("\"function\"") || stdout.contains("\"type\"") || stdout == "[]\n"
}

fn compile_and_check_ir_with_target(source_path: &str, target: &str, expected_triple: &str) -> bool {
    let output = Command::new(axiomc_path())
        .args(["--target", target, "--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(expected_triple)
}

fn compile_ir(source_path: &str) -> Option<String> {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

// ============================================================================
// E2E: Native Compilation + Exit Code Verification
// ============================================================================

#[test]
fn e2e_demo_float_returns_30() {
    assert_eq!(compile_and_run("examples\\demo_float.ax"), Some(30));
}

#[test]
fn e2e_diff_test_returns_42() {
    assert_eq!(compile_and_run("examples\\diff_test.ax"), Some(42));
}

#[test]
fn e2e_ownership_compiles() {
    assert!(compile_and_run("examples\\phase1_ownership.ax").is_some());
}

#[test]
fn e2e_derive_compiles() {
    assert!(compile_and_run("examples\\phase1_derive.ax").is_some());
}

#[test]
fn e2e_contracts_compiles() {
    assert!(compile_and_run("examples\\phase1_contracts.ax").is_some());
}

#[test]
fn e2e_error_compiles() {
    assert!(compile_and_run("examples\\phase1_error.ax").is_some());
}

#[test]
fn e2e_generics_compiles() {
    assert!(compile_and_run("examples\\phase1_generics.ax").is_some());
}

#[test]
fn e2e_modules_compiles() {
    assert!(compile_and_run("examples\\phase1_modules.ax").is_some());
}

#[test]
fn e2e_async_compiles() {
    assert!(compile_and_run("examples\\phase1_async.ax").is_some());
}

#[test]
fn e2e_full_compiles() {
    assert!(compile_and_run("examples\\phase1_full.ax").is_some());
}

#[test]
fn e2e_enum_compiles() {
    assert!(compile_and_run("examples\\phase1_enum.ax").is_some());
}

#[test]
fn e2e_interface_compiles() {
    assert!(compile_and_run("examples\\phase1_interface.ax").is_some());
}

#[test]
fn e2e_derive_enum_compiles() {
    assert!(compile_and_run("examples\\phase1_derive_enum.ax").is_some());
}

#[test]
fn e2e_hardening_compiles() {
    assert!(compile_and_run("examples\\phase1_hardening.ax").is_some());
}

#[test]
fn e2e_stress_compiles() {
    assert!(compile_and_run("examples\\phase1_stress.ax").is_some());
}

#[test]
fn e2e_selfhost_sim_compiles() {
    assert!(compile_and_run("examples\\phase1_selfhost.ax").is_some());
}

#[test]
fn e2e_async_spawn_compiles() {
    assert!(compile_and_run("examples\\phase1_async_spawn.ax").is_some());
}

// ============================================================================
// E2E: Stress Tests
// ============================================================================

#[test]
fn e2e_stress_derive_50field() {
    assert!(compile_and_run("examples\\stress_derive_50field.ax").is_some());
}

#[test]
fn e2e_stress_borrow_10level() {
    assert!(compile_and_run("examples\\stress_borrow_10level.ax").is_some());
}

#[test]
fn e2e_stress_generic_5chain() {
    assert!(compile_and_run("examples\\stress_generic_5chain.ax").is_some());
}

#[test]
fn e2e_stress_float_matrix() {
    assert!(compile_and_run("examples\\stress_float_matrix.ax").is_some());
}

// ============================================================================
// E2E: WASM Target
// ============================================================================

#[test]
fn e2e_wasm_demo_float() {
    assert!(compile_wasm("examples\\demo_float.ax"));
}

#[test]
fn e2e_wasm_ownership() {
    assert!(compile_wasm("examples\\phase1_ownership.ax"));
}

#[test]
fn e2e_wasm_generics() {
    assert!(compile_wasm("examples\\phase1_generics.ax"));
}

#[test]
fn e2e_wasm_diff_test() {
    assert!(compile_wasm("examples\\diff_test.ax"));
}

#[test]
fn e2e_wasm_async_spawn() {
    assert!(compile_wasm("examples\\phase1_async_spawn.ax"));
}

// ============================================================================
// E2E: IR Verification
// ============================================================================

#[test]
fn e2e_ir_contains_define() {
    assert!(compile_and_check_ir("examples\\demo_float.ax", "define i64 @main"));
}

#[test]
fn e2e_ir_contains_main_ret() {
    assert!(compile_and_check_ir("examples\\diff_test.ax", "ret i64 42"));
}

#[test]
fn e2e_ir_contains_struct() {
    assert!(compile_and_check_ir("examples\\phase1_derive.ax", "%struct.Point"));
}

#[test]
fn e2e_ir_contains_trap() {
    assert!(compile_and_check_ir("examples\\phase1_contracts.ax", "@llvm.trap"));
}

#[test]
fn e2e_ir_contains_generic_monomorph() {
    assert!(compile_and_check_ir("examples\\phase1_generics.ax", "define i64 @wrap_Int"));
}

#[test]
fn e2e_ir_contains_ownership() {
    assert!(compile_and_check_ir("examples\\phase1_ownership.ax", "define i64 @take_ownership"));
}

#[test]
fn e2e_ir_contains_module_fn() {
    assert!(compile_and_check_ir("examples\\phase1_modules.ax", "define i64 @add"));
}

// ============================================================================
// E2E: CLI Flags
// ============================================================================

#[test]
fn e2e_diagnostics_json() {
    assert!(compile_diagnostics_json("examples\\demo_float.ax"));
}

#[test]
fn e2e_dump_contracts() {
    assert!(compile_dump_contracts("examples\\phase1_contracts.ax"));
}

#[test]
fn e2e_emit_ir_flag() {
    let ir = compile_ir("examples\\diff_test.ax");
    assert!(ir.is_some());
    assert!(ir.unwrap().contains("define i64 @main"));
}

// ============================================================================
// E2E: Cross-Target Triples (IR emission only, no native compile needed)
// ============================================================================

#[test]
fn e2e_target_wasm_triple() {
    assert!(compile_and_check_ir_with_target("examples\\demo_float.ax", "wasm", "wasm32-unknown-unknown"));
}

#[test]
fn e2e_target_arm_triple() {
    assert!(compile_and_check_ir_with_target("examples\\demo_float.ax", "arm", "aarch64-unknown-linux-gnu"));
}

#[test]
fn e2e_target_riscv_triple() {
    assert!(compile_and_check_ir_with_target("examples\\demo_float.ax", "riscv", "riscv64gc-unknown-linux-gnu"));
}

// ============================================================================
// E2E: Selfhost Pipeline (compile only, IR emission)
// ============================================================================

#[test]
fn e2e_selfhost_lexer_compiles() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiom-lexer.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/axiom-lexer.ax should compile");
}

#[test]
fn e2e_selfhost_parser_compiles() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiom-parser.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/axiom-parser.ax should compile");
}

#[test]
fn e2e_selfhost_check_compiles() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiom-check.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/axiom-check.ax should compile");
}

#[test]
fn e2e_selfhost_codegen_compiles() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiom-codegen.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/axiom-codegen.ax should compile");
}

#[test]
fn e2e_selfhost_compiler_module() {
    let ir = compile_ir("selfhost\\axiomc.ax").expect("selfhost/axiomc.ax should compile to IR");
    assert!(ir.contains("define i64 @main"), "selfhost compiler should have main");
}

#[test]
fn e2e_no_contracts_flag() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "--no-contracts", "examples\\phase1_contracts.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "contracts should compile without contract checks");
}

#[test]
fn e2e_target_wasm_selfhost_lexer() {
    let output = Command::new(axiomc_path())
        .args(["--target", "wasm", "--emit-ir", "selfhost\\axiom-lexer.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost lexer should compile to WASM IR");
}

// ============================================================================
// E2E: Extern Runtime (axiom_runtime.c)
// ============================================================================

#[test]
fn e2e_runtime_c_exists() {
    assert!(
        project_root().join("stdlib\\runtime\\axiom_runtime.c").exists()
            || project_root().join("stdlib/runtime/axiom_runtime.c").exists(),
        "stdlib/runtime/axiom_runtime.c should exist"
    );
}

#[test]
fn e2e_selfhost_v091_compiles() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiomc_v091.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/axiomc_v091.ax should compile to IR");
}

#[test]
fn e2e_selfhost_v091_has_main() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiomc_v091.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v091 should have main");
}

#[test]
fn e2e_selfhost_v094_compiles() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiomc_v094.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/axiomc_v094.ax should compile to IR");
}

#[test]
fn e2e_selfhost_v094_has_main() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiomc_v094.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v094 should have main");
}

#[test]
fn e2e_selfhost_v094_contains_extern_decls() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiomc_v094.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("@axiom_ir_define_s"), "v094 IR should declare axiom_ir_define_s");
    assert!(stdout.contains("@axiom_ir_call_fn"), "v094 IR should declare axiom_ir_call_fn");
    assert!(stdout.contains("@axiom_ir_call_arg_lit"), "v094 IR should declare axiom_ir_call_arg_lit");
    assert!(stdout.contains("@axiom_ir_param_int"), "v094 IR should declare axiom_ir_param_int");
    assert!(stdout.contains("@axiom_ir_param_double"), "v094 IR should declare axiom_ir_param_double");
    assert!(stdout.contains("@axiom_ir_fmul"), "v094 IR should declare axiom_ir_fmul");
}

#[test]
fn e2e_selfhost_v094_contains_codegen_fn() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiomc_v094.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define void @emit_demo_float"), "v094 should define emit_demo_float");
}

#[test]
fn e2e_runtime_ir_declares_externs() {
    let output = Command::new(axiomc_path())
        .args(["--emit-ir", "selfhost\\axiomc_v091.ax"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("@axiom_read_file"), "IR should declare axiom_read_file");
    assert!(stdout.contains("@axiom_file_size"), "IR should declare axiom_file_size");
    assert!(stdout.contains("@axiom_free"), "IR should declare axiom_free");
}
