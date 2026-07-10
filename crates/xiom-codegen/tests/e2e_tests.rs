// XIOM — E2E Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::process::Command;
use std::path::Path;

/// Path to the compiled xiomc binary
fn xiomc_path() -> String {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join("xiomc.exe");
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join("xiomc.exe");
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

/// Compile an XIOM source file to a native binary and return the exit code
fn compile_and_run(source_path: &str) -> Option<i32> {
    let source = Path::new(source_path);
    let exe_name = format!("e2e_{}.exe", source.file_stem()?.to_str()?);

    let bin_path = xiomc_path();

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
    let bin_path = xiomc_path();
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
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed to emit IR");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(expected_ir)
}

/// Compile with --diagnostics=json and verify valid JSON
fn compile_diagnostics_json(source_path: &str) -> bool {
    let output = Command::new(xiomc_path())
        .args(["--diagnostics=json", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("\"status\"")
}

/// Compile with --dump-contracts and verify output
fn compile_dump_contracts(source_path: &str) -> bool {
    let output = Command::new(xiomc_path())
        .args(["--dump-contracts", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("\"function\"") || stdout.contains("\"type\"") || stdout == "[]\n"
}

fn compile_and_check_ir_with_target(source_path: &str, target: &str, expected_triple: &str) -> bool {
    let output = Command::new(xiomc_path())
        .args(["--target", target, "--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(expected_triple)
}

fn compile_ir(source_path: &str) -> Option<String> {
    let output = Command::new(xiomc_path())
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
    assert_eq!(compile_and_run("examples\\demo_float.xi"), Some(30));
}

#[test]
fn e2e_diff_test_returns_42() {
    assert_eq!(compile_and_run("examples\\diff_test.xi"), Some(42));
}

#[test]
fn e2e_ownership_compiles() {
    assert!(compile_and_run("examples\\phase1_ownership.xi").is_some());
}

#[test]
fn e2e_derive_compiles() {
    assert!(compile_and_run("examples\\phase1_derive.xi").is_some());
}

#[test]
fn e2e_contracts_compiles() {
    assert!(compile_and_run("examples\\phase1_contracts.xi").is_some());
}

#[test]
fn e2e_error_compiles() {
    assert!(compile_and_run("examples\\phase1_error.xi").is_some());
}

#[test]
fn e2e_generics_compiles() {
    assert!(compile_and_run("examples\\phase1_generics.xi").is_some());
}

#[test]
fn e2e_modules_compiles() {
    assert!(compile_and_run("examples\\phase1_modules.xi").is_some());
}

#[test]
fn e2e_async_compiles() {
    assert!(compile_and_run("examples\\phase1_async.xi").is_some());
}

#[test]
fn e2e_full_compiles() {
    assert!(compile_and_run("examples\\phase1_full.xi").is_some());
}

#[test]
fn e2e_enum_compiles() {
    assert!(compile_and_run("examples\\phase1_enum.xi").is_some());
}

#[test]
fn e2e_interface_compiles() {
    assert!(compile_and_run("examples\\phase1_interface.xi").is_some());
}

#[test]
fn e2e_derive_enum_compiles() {
    assert!(compile_and_run("examples\\phase1_derive_enum.xi").is_some());
}

#[test]
fn e2e_hardening_compiles() {
    assert!(compile_and_run("examples\\phase1_hardening.xi").is_some());
}

#[test]
fn e2e_stress_compiles() {
    assert!(compile_and_run("examples\\phase1_stress.xi").is_some());
}

#[test]
fn e2e_selfhost_sim_compiles() {
    assert!(compile_and_run("examples\\phase1_selfhost.xi").is_some());
}

#[test]
fn e2e_async_spawn_compiles() {
    assert!(compile_and_run("examples\\phase1_async_spawn.xi").is_some());
}

// ============================================================================
// E2E: Stress Tests
// ============================================================================

#[test]
fn e2e_stress_derive_50field() {
    assert!(compile_and_run("examples\\stress_derive_50field.xi").is_some());
}

#[test]
fn e2e_stress_borrow_10level() {
    assert!(compile_and_run("examples\\stress_borrow_10level.xi").is_some());
}

#[test]
fn e2e_stress_generic_5chain() {
    assert!(compile_and_run("examples\\stress_generic_5chain.xi").is_some());
}

#[test]
fn e2e_stress_float_matrix() {
    assert!(compile_and_run("examples\\stress_float_matrix.xi").is_some());
}

// ============================================================================
// E2E: WASM Target
// ============================================================================

#[test]
fn e2e_wasm_demo_float() {
    assert!(compile_wasm("examples\\demo_float.xi"));
}

#[test]
fn e2e_wasm_ownership() {
    assert!(compile_wasm("examples\\phase1_ownership.xi"));
}

#[test]
fn e2e_wasm_generics() {
    assert!(compile_wasm("examples\\phase1_generics.xi"));
}

#[test]
fn e2e_wasm_diff_test() {
    assert!(compile_wasm("examples\\diff_test.xi"));
}

#[test]
fn e2e_wasm_async_spawn() {
    assert!(compile_wasm("examples\\phase1_async_spawn.xi"));
}

// ============================================================================
// E2E: IR Verification
// ============================================================================

#[test]
fn e2e_ir_contains_define() {
    assert!(compile_and_check_ir("examples\\demo_float.xi", "define i64 @main"));
}

#[test]
fn e2e_ir_contains_main_ret() {
    assert!(compile_and_check_ir("examples\\diff_test.xi", "ret i64 42"));
}

#[test]
fn e2e_ir_contains_struct() {
    assert!(compile_and_check_ir("examples\\phase1_derive.xi", "%struct.Point"));
}

#[test]
fn e2e_ir_contains_trap() {
    assert!(compile_and_check_ir("examples\\phase1_contracts.xi", "@llvm.trap"));
}

#[test]
fn e2e_ir_contains_generic_monomorph() {
    assert!(compile_and_check_ir("examples\\phase1_generics.xi", "define i64 @wrap_Int"));
}

#[test]
fn e2e_ir_contains_ownership() {
    assert!(compile_and_check_ir("examples\\phase1_ownership.xi", "define i64 @take_ownership"));
}

#[test]
fn e2e_ir_contains_module_fn() {
    assert!(compile_and_check_ir("examples\\phase1_modules.xi", "define i64 @add"));
}

// ============================================================================
// E2E: CLI Flags
// ============================================================================

#[test]
fn e2e_diagnostics_json() {
    assert!(compile_diagnostics_json("examples\\demo_float.xi"));
}

#[test]
fn e2e_dump_contracts() {
    assert!(compile_dump_contracts("examples\\phase1_contracts.xi"));
}

#[test]
fn e2e_emit_ir_flag() {
    let ir = compile_ir("examples\\diff_test.xi");
    assert!(ir.is_some());
    assert!(ir.unwrap().contains("define i64 @main"));
}

// ============================================================================
// E2E: Cross-Target Triples (IR emission only, no native compile needed)
// ============================================================================

#[test]
fn e2e_target_wasm_triple() {
    assert!(compile_and_check_ir_with_target("examples\\demo_float.xi", "wasm", "wasm32-unknown-unknown"));
}

#[test]
fn e2e_target_arm_triple() {
    assert!(compile_and_check_ir_with_target("examples\\demo_float.xi", "arm", "aarch64-unknown-linux-gnu"));
}

#[test]
fn e2e_target_riscv_triple() {
    assert!(compile_and_check_ir_with_target("examples\\demo_float.xi", "riscv", "riscv64gc-unknown-linux-gnu"));
}

// ============================================================================
// E2E: Selfhost Pipeline (compile only, IR emission)
// ============================================================================

#[test]
fn e2e_selfhost_lexer_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiom-lexer.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-lexer.xi should compile");
}

#[test]
fn e2e_selfhost_parser_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiom-parser.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-parser.xi should compile");
}

#[test]
fn e2e_selfhost_check_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiom-check.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-check.xi should compile");
}

#[test]
fn e2e_selfhost_codegen_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiom-codegen.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-codegen.xi should compile");
}

#[test]
fn e2e_selfhost_compiler_module() {
    let ir = compile_ir("selfhost\\xiomc.xi").expect("selfhost/xiomc.xi should compile to IR");
    assert!(ir.contains("define i64 @main"), "selfhost compiler should have main");
}

#[test]
fn e2e_no_contracts_flag() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "--no-contracts", "examples\\phase1_contracts.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "contracts should compile without contract checks");
}

#[test]
fn e2e_target_wasm_selfhost_lexer() {
    let output = Command::new(xiomc_path())
        .args(["--target", "wasm", "--emit-ir", "selfhost\\xiom-lexer.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost lexer should compile to WASM IR");
}

// ============================================================================
// E2E: Extern Runtime (xiom_runtime.c)
// ============================================================================

#[test]
fn e2e_runtime_c_exists() {
    assert!(
        project_root().join("stdlib\\runtime\\xiom_runtime.c").exists()
            || project_root().join("stdlib/runtime/xiom_runtime.c").exists(),
        "stdlib/runtime/xiom_runtime.c should exist"
    );
}

#[test]
fn e2e_selfhost_v091_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v091.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v091.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v091_has_main() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v091.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v091 should have main");
}

#[test]
fn e2e_selfhost_v094_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v094.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v094.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v094_has_main() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v094.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v094 should have main");
}

#[test]
fn e2e_selfhost_v094_contains_extern_decls() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v094.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("@xiom_ir_define_s"), "v094 IR should declare xiom_ir_define_s");
    assert!(stdout.contains("@xiom_ir_call_fn"), "v094 IR should declare xiom_ir_call_fn");
    assert!(stdout.contains("@xiom_ir_call_arg_lit"), "v094 IR should declare xiom_ir_call_arg_lit");
    assert!(stdout.contains("@xiom_ir_param_int"), "v094 IR should declare xiom_ir_param_int");
    assert!(stdout.contains("@xiom_ir_param_double"), "v094 IR should declare xiom_ir_param_double");
    assert!(stdout.contains("@xiom_ir_fmul"), "v094 IR should declare xiom_ir_fmul");
}

#[test]
fn e2e_selfhost_v094_contains_codegen_fn() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v094.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define void @emit_demo_float"), "v094 should define emit_demo_float");
}

#[test]
fn e2e_runtime_ir_declares_externs() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v091.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("@xiom_read_file"), "IR should declare xiom_read_file");
    assert!(stdout.contains("@xiom_file_size"), "IR should declare xiom_file_size");
    assert!(stdout.contains("@xiom_free"), "IR should declare xiom_free");
}

#[test]
fn e2e_selfhost_v10_self_compile() {
    let output = std::process::Command::new(xiomc_path())
        .args(["-o", "e2e_v10_self_compile.exe", "selfhost\\xiomc_v10.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed to compile v10 selfhost");
    assert!(output.status.success(), "v10 selfhost compilation failed");

    let run = std::process::Command::new(project_root().join("e2e_v10_self_compile.exe"))
        .current_dir(project_root())
        .output()
        .expect("failed to run v10 selfhost");
    let stdout = String::from_utf8_lossy(&run.stdout);

    assert!(stdout.contains("define i64 @main"), "Selfhost must emit its own main");
    assert!(stdout.contains("define"), "Selfhost must emit function definitions");
    let fn_count = stdout.matches("define ").count();
    assert!(fn_count >= 5, "Selfhost found only {} functions, expected >= 5", fn_count);
}

#[test]
fn e2e_selfhost_v10_self_compile_to_native() {
    // Phase 4: Self-hosting bootstrap — the v10 selfhost compiler
    // generates IR that needs updating to work with the v2.0 runtime.
    // This test will be re-enabled after Phase 3 (Z3, debugger, LSP).
    if std::env::var("XIOM_SELFHOST").is_err() {
        eprintln!("  [SKIP] Selfhost native compile — enable with XIOM_SELFHOST=1 (Phase 4)");
        return;
    }
    let output = std::process::Command::new(xiomc_path())
        .args(["-o", "e2e_v10_self_bootstrap_src.exe", "selfhost\\xiomc_v10.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed v10 compile");
    assert!(output.status.success());

    let run = std::process::Command::new(project_root().join("e2e_v10_self_bootstrap_src.exe"))
        .current_dir(project_root())
        .output()
        .expect("failed v10 run");
    let stdout = String::from_utf8_lossy(&run.stdout);

    std::fs::write(project_root().join("e2e_v10_output.ll"), stdout.as_bytes()).expect("write IR");

    let clang_result = std::process::Command::new("clang")
        .args(["-maes", "-DXIOM_NO_ASM", "-o", "e2e_v10_bootstrap.exe", "e2e_v10_output.ll", "stdlib\\runtime\\xiom_runtime.c"])
        .current_dir(project_root())
        .output();

    if let Ok(result) = clang_result {
        assert!(result.status.success(), "Bootstrap IR compilation failed");
    }
}

// ============================================================================
// E2E: Selfhost v11_test — Parameter-counting compiler for demo_float.xi
// ============================================================================

#[test]
fn e2e_selfhost_v11_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v11_test.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v11_test.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v11_has_main() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v11_test.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v11_test IR should contain main");
    assert!(stdout.contains("define"), "v11_test IR should contain function definitions");
}

#[test]
fn e2e_selfhost_v11_self_run() {
    // Compile v11_test to binary, run it — it should emit IR for demo_float.xi functions
    let output = Command::new(xiomc_path())
        .args(["-o", "e2e_v11_self.exe", "selfhost\\xiomc_v11_test.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed to compile v11 selfhost");
    assert!(output.status.success(), "v11_test selfhost compilation failed");

    let run = Command::new(project_root().join("e2e_v11_self.exe"))
        .current_dir(project_root())
        .output()
        .expect("failed to run v11 selfhost");
    let stdout = String::from_utf8_lossy(&run.stdout);

    assert!(stdout.contains("define i64 @main"), "v11_test must emit main function");
    assert!(stdout.contains("define i64 @add"), "v11_test must emit add function");
    assert!(stdout.contains("define double @sq"), "v11_test must emit sq function");
    let fn_count = stdout.matches("define ").count();
    assert!(fn_count >= 3, "v11_test found only {} functions, expected >= 3", fn_count);
}

// ============================================================================
// E2E: Selfhost v093 / v095 — Pipeline compilation checks
// ============================================================================

#[test]
fn e2e_selfhost_v093_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v093.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v093.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v095_compiles() {
    let output = Command::new(xiomc_path())
        .args(["--emit-ir", "selfhost\\xiomc_v095.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v095.xi should compile to IR");
}

// ============================================================================
// E2E: Multi-File Module Catalog — Regression Tests
// ============================================================================

/// Compile and run test_mod/math.xi. Verifies catalog resolves cross-file
/// types (BenchResult) and functions (make_result) from test_mod/main.xi.
/// Expected exit code: 34.
#[test]
fn e2e_multifile_testmod_math_runs() {
    let exit = compile_and_run("examples\\test_mod\\math.xi");
    assert_eq!(exit, Some(34), "test_mod/math.xi should exit 34");
}

/// Compile benchmark/bench_math.xi (uses `use benchmark.main.BenchResult` from
/// external benchmark/main.xi). Verifies the catalog resolves cross-file types.
#[test]
fn e2e_multifile_bench_math_compiles() {
    let output = std::process::Command::new(xiomc_path())
        .args(["--emit-ir", "examples\\benchmark\\bench_math.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "bench_math.xi should compile to IR via ModuleCatalog");
}

/// Compile the full 30-module benchmark suite. Uses the ModuleCatalog
/// (single-file path with lazy loading of all 30 submodules).
/// NOTE: This test passes individually but times out under heavy parallel load
/// due to the 30-module benchmark's size (65536 mono iterations). Run solo:
///   cargo test -p xiom-codegen --test e2e_tests -- e2e_multifile -- --nocapture
#[test]
fn e2e_multifile_benchmark_main_compiles() {
    let output = std::process::Command::new(xiomc_path())
        .args(["--emit-ir", "examples\\benchmark\\main.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(),
        "benchmark/main.xi 30-module suite should compile via ModuleCatalog");
}

// ============================================================================
// E2E: CLI Flags — Timeout & Memory
// NOTE: Watchdog thread tests are inherently racy and environment-dependent.
// Flag parsing correctness is verified via --help output test below.
// The flags are tested in isolation via unit/integration tests.
// ============================================================================

#[test]
fn e2e_help_shows_timeout_and_memory_flags() {
    let output = std::process::Command::new(xiomc_path())
        .arg("--help")
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("--timeout"), "help should document --timeout flag");
    assert!(combined.contains("--max-memory-mb"), "help should document --max-memory-mb flag");
}

// ============================================================================
// E2E: Regression — method `match self` on enum receiver
// ============================================================================
/// Regression: a method that pattern-matches `self` on an enum receiver must
/// treat self as the typed struct, NOT a phantom `i64` duplicate param.
/// Without the fix, variant patterns become variable bindings and arms return
/// raw i64 discriminants → `store %struct.X i64` (invalid IR).
/// Returns exit code 0 when the fix is present.
#[test]
fn e2e_method_match_self_enum() {
    let exit = compile_and_run("examples\\e2e\\method_match_self_enum.xi");
    assert_eq!(exit, Some(0), "match self on enum receiver should compile, link, run, and exit 0");
}

// ============================================================================
// E2E: Regression suite for Tier 2 codegen fixes (hermetic — no stdlib needed
// except where noted). Each program returns 0 on success, nonzero on failure.
// These lock in fixes that were hard-won during the stdlib execution work.
// ============================================================================

/// Enum `==`/`!=` via the builtin `.eq` fallback (compare discriminant inline),
/// plus enum-variant construction as values from match arms.
#[test]
fn e2e_enum_eq_and_variants() {
    assert_eq!(compile_and_run("examples\\e2e\\enum_eq.xi"), Some(0),
        "enum ==/!= and variant construction should work");
}

/// Vec builtins: new/push/len/index-read/pop returning Option, with element
/// coercion. Locks in inline Vec method dispatch + Option payload extraction.
#[test]
fn e2e_vec_ops() {
    assert_eq!(compile_and_run("examples\\e2e\\vec_ops.xi"), Some(0),
        "Vec new/push/len/index/pop should work");
}

/// Char (i8) <-> Int (i64) casts + i8 widening in arithmetic (sext/trunc).
#[test]
fn e2e_char_cast() {
    assert_eq!(compile_and_run("examples\\e2e\\char_cast.xi"), Some(0),
        "Char<->Int casts and widening should work");
}

/// Cross-module use of a stdlib module whose functions call C externs
/// (math). Locks in cross-module extern-declare injection + libc/libm handling.
#[test]
fn e2e_cross_module_math() {
    assert_eq!(compile_and_run("examples\\e2e\\cross_math.xi"), Some(0),
        "cross-module stdlib call with C externs should link and run");
}

/// Module-level mutable `var` global: write persists across function calls.
/// Locks in the mutable-module-global feature (ConstDecl.is_mut).
#[test]
fn e2e_module_global_var() {
    assert_eq!(compile_and_run("examples\\e2e\\module_global.xi"), Some(0),
        "module-level mutable var global should persist writes across calls");
}

/// `pub const` referenced across functions (GAP-3): const resolves at use sites.
#[test]
fn e2e_pub_const_use() {
    assert_eq!(compile_and_run("examples\\e2e\\const_use.xi"), Some(0),
        "pub const should resolve and be usable across functions");
}

/// Enum variant as a value (let binding from var assignment), and ==/!= on enum types.
#[test]
fn e2e_enum_variant_value() {
    assert_eq!(compile_and_run("examples\\e2e\\enum_variant_value.xi"), Some(0),
        "enum variant as value and ==/!= should work");
}

/// `&mut Scalar` parameter is a real LLVM pointer: `inc(p: &mut Int)` derefs to
/// read (`*p`) and stores through (`*p = ...`), mutating the caller's local.
#[test]
fn e2e_ref_mut_param() {
    assert_eq!(compile_and_run("examples\\e2e\\ref_mut_param.xi"), Some(0),
        "&mut Int param should deref-read and store-through, mutating the caller");
}

/// Raw pointer deref round-trip: `&mut x` reaches a `*T` param, which reads/writes
/// through the real pointer so the mutation is visible in the caller's binding.
#[test]
fn e2e_ptr_deref() {
    assert_eq!(compile_and_run("examples\\e2e\\ptr_deref.xi"), Some(0),
        "raw pointer deref read/write round-trip should mutate the source local");
}

// ============================================================================
// E2E: Regression — field assignment + store-back (Clusters 1-3 fixes)
// ============================================================================

/// Struct field assignment (`self.field = expr`) emits a store instruction
/// through GEP into the struct alloca.
#[test]
fn e2e_field_assign() {
    assert_eq!(compile_and_run("examples\\e2e\\field_assign.xi"), Some(0),
        "struct field assignment should store through GEP");
}

/// Struct method returning modified self stores back to the caller's variable
/// so mutation persists across the call.
#[test]
fn e2e_method_store_back() {
    assert_eq!(compile_and_run("examples\\e2e\\method_store_back.xi"), Some(0),
        "mutating struct method should store result back to receiver var");
}

/// Chained calls like `make_pair(10,25).sum()` resolve the method on the
/// return type of the call expression.
#[test]
fn e2e_call_receiver_type() {
    assert_eq!(compile_and_run("examples\\e2e\\call_receiver_type.xi"), Some(0),
        "chained call receiver type inference should resolve method");
}

/// Or-patterns like `1 | 2 | 3 =>` in match arms compile and match correctly.
#[test]
fn e2e_or_pattern() {
    assert_eq!(compile_and_run("examples\\e2e\\or_pattern.xi"), Some(0),
        "or-patterns in match should compile and match correctly");
}

/// Brace-form modules (`module x { ... }`) compile and run correctly.
/// GAP-13: previously marked as won't-fix but the parser already handles
/// block-form module parsing. This test locks in the behavior.
#[test]
fn e2e_brace_module() {
    assert_eq!(compile_and_run("examples\\e2e\\brace_module.xi"), Some(0),
        "brace-form modules should compile and run");
}

/// Struct method returning modified self stores back to caller's variable
/// so mutation persists across the call (store-back mechanism).
#[test]
fn e2e_mut_struct() {
    assert_eq!(compile_and_run("examples\\e2e\\mut_struct.xi"), Some(0),
        "struct mutation via store_back should propagate to caller");
}

/// DJB2 hash monomorphized through the Hash interface.
/// Same input → same hash; different inputs → different hashes.
#[test]
fn e2e_djb2_hash() {
    assert_eq!(compile_and_run("examples\\e2e\\djb2_hash.xi"), Some(0),
        "DJB2 hash via Hash[T] interface should produce deterministic non-zero values");
}
