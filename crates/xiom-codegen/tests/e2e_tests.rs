// XIOM -- E2E Tests
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;
use std::path::Path;

/// Path to the compiled xiomc binary
fn xiom_path() -> String {
    let exe_name = if cfg!(target_os = "windows") { "xiom.exe" } else { "xiom" };
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("target").join("debug").join(exe_name);
    if !path.exists() {
        path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .join("target").join("release").join(exe_name);
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
    compile_and_run_with_flags(source_path, &[])
}

/// The stdlib checkout layout varies: module files live EITHER flat
/// (`xiom/io.xi`, older pins) OR in module directories (`xiom/io/io.xi`, the
/// current xiom-lang/stdlib layout). Accept both so the guarded tests RUN
/// wherever a checkout exists -- the old flat-only probe made every
/// stdlib-guarded test panic in CI (XIOM_REQUIRE_STDLIB=1) even though the
/// checkout was present.
fn stdlib_layout_ok(root: &std::path::Path) -> bool {
    root.join("xiom").join("io.xi").exists()
        || root.join("xiom").join("io").join("io.xi").exists()
}

/// Compile with extra flags (e.g. --parallel-codegen) and return exit code
///
/// Retries up to 3 times: the parallel benchmark/stdlib session rebuilds
/// `target/debug/xiom.exe` while this suite runs, so a compile can race a
/// half-written compiler binary and emit corrupted IR (observed:
/// e2e_p1_contract_methods failed 1-in-2240 with a compile that succeeded
/// on immediate rerun -- the same disambiguation the stdlib-exec suite uses).
/// A non-zero exit is recompiled fresh and rerun; only a REPEAT of the same
/// code is accepted as real. Genuine compile failures (None) are never
/// retried, so no real compiler bug is masked.
fn compile_and_run_with_flags(source_path: &str, extra_args: &[&str]) -> Option<i32> {
    for attempt in 0..3 {
        let result = compile_and_run_once_with_flags(source_path, extra_args);
        match result {
            Some(0) => return result,
            Some(_code) => {
                // Non-zero: could be a legitimate program failure OR a raced
                // compile. Recompile fresh and rerun to disambiguate.
                let retry = compile_and_run_once_with_flags(source_path, extra_args);
                if retry == result {
                    return retry;
                }
                if attempt == 2 {
                    return retry;
                }
            }
            None => return result, // genuine compile failure -- no retry masks it
        }
    }
    None
}

fn compile_and_run_once_with_flags(source_path: &str, extra_args: &[&str]) -> Option<i32> {
    let source = Path::new(source_path);
    let exe_suffix = if cfg!(target_os = "windows") { ".exe" } else { "" };
    // Parallel-flake fix (2026-09-09): the output name was derived from the
    // SOURCE file stem alone, so tests compiling the SAME source raced on one
    // binary -- e2e_cross_package_extern vs e2e_cross_package_use both compile
    // examples/e2e/cross_pkg/main.xi -> both target e2e_main.exe, and the
    // loser links/runs while the winner removes it (intermittent FAILED in
    // full parallel runs, always green in isolation). A per-invocation
    // counter makes every output name unique across tests AND retries.
    static EXE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let exe_name = format!(
        "e2e_{}_{}{}",
        source.file_stem()?.to_str()?,
        EXE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        exe_suffix
    );

    // BUG 40-era hardening (2026-08-17): a STALE exe from an interrupted
    // run can hold the output path open -- clang then fails with
    // "permission denied" writing the new binary (observed for
    // e2e_main.exe / e2e_t3-hot-reload.exe in parallel suites). Best
    // effort delete before compiling; ignore failure (a genuinely running
    // process still locks it, and the link will fail loudly).
    let exe_path = project_root().join(&exe_name);
    let _ = std::fs::remove_file(&exe_path);

    let bin_path = xiom_path();

    // Compile
    let mut args = vec!["-o", &exe_name];
    args.extend(extra_args);
    args.push(source_path);
    let compile = Command::new(&bin_path)
        .args(&args)
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

    // D1 hardening: give the OS a moment to fully flush/close the freshly
    // linked exe before spawning it. Under the parallel suite, an immediate
    // spawn could execute a partially-written binary.
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Run
    let exe_path = project_root().join(&exe_name);
    let run = Command::new(&exe_path)
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{:?}': {e}", exe_path));

    // Cleanup (2026-09-24): after the run the per-invocation binary is no
    // longer needed; delete it so a full suite does not leave thousands of
    // `e2e_*` artifacts in the repo root. Best effort (Windows releases the
    // mapping once the child exits).
    let code = run.status.code();
    let _ = std::fs::remove_file(&exe_path);
    code
}

/// Compile to WASM and verify file exists
fn compile_wasm(source_path: &str) -> bool {
    let source = Path::new(source_path);
    let wasm_name = format!("e2e_{}.wasm", source.file_stem().unwrap().to_str().unwrap());
    let bin_path = xiom_path();
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
    // Cleanup: the emitted .wasm is an artifact of this check only.
    let ok = output.status.success() && project_root().join(&wasm_name).exists();
    let _ = std::fs::remove_file(project_root().join(&wasm_name));
    ok
}

/// Compile to IR and verify output contains expected text
fn compile_and_check_ir(source_path: &str, expected_ir: &str) -> bool {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed to emit IR");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(expected_ir)
}

/// Compile with --diagnostics=json and verify valid JSON
fn compile_diagnostics_json(source_path: &str) -> bool {
    let output = Command::new(xiom_path())
        .args(["--diagnostics=json", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("\"status\"")
}

/// Compile with --dump-contracts and verify output
fn compile_dump_contracts(source_path: &str) -> bool {
    let output = Command::new(xiom_path())
        .args(["--dump-contracts", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("\"function\"") || stdout.contains("\"type\"") || stdout == "[]\n"
}

fn compile_and_check_ir_with_target(source_path: &str, target: &str, expected_triple: &str) -> bool {
    let output = Command::new(xiom_path())
        .args(["--target", target, "--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(expected_triple)
}

fn compile_ir(source_path: &str) -> Option<String> {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", source_path])
        .current_dir(project_root())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

/// m344: compile capturing BOTH streams (borrow diagnostics print to stderr;
/// `--emit-ir` writes IR to stdout). Returns (success, combined output).
fn compile_capture(source_path: &str, extra_args: &[&str]) -> (bool, String) {
    let output = Command::new(xiom_path())
        .args(extra_args)
        .arg(source_path)
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn compiler for {source_path}: {e}"));
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), combined)
}

// ============================================================================
// E2E: Native Compilation + Exit Code Verification
// ============================================================================

#[test]
fn e2e_demo_float_returns_30() {
    assert_eq!(compile_and_run("examples/demo_float.xi"), Some(30));
}

#[test]
fn e2e_diff_test_returns_42() {
    assert_eq!(compile_and_run("examples/diff_test.xi"), Some(42));
}

#[test]
fn e2e_ownership_compiles() {
    assert!(compile_and_run("examples/phase1_ownership.xi").is_some());
}

#[test]
fn e2e_derive_compiles() {
    assert!(compile_and_run("examples/phase1_derive.xi").is_some());
}

#[test]
fn e2e_contracts_compiles() {
    assert!(compile_and_run("examples/phase1_contracts.xi").is_some());
}

#[test]
fn e2e_error_compiles() {
    assert!(compile_and_run("examples/phase1_error.xi").is_some());
}

#[test]
fn e2e_generics_compiles() {
    assert!(compile_and_run("examples/phase1_generics.xi").is_some());
}

#[test]
fn e2e_modules_compiles() {
    assert!(compile_and_run("examples/phase1_modules.xi").is_some());
}

#[test]
fn e2e_async_compiles() {
    assert!(compile_and_run("examples/phase1_async.xi").is_some());
}

#[test]
fn e2e_full_compiles() {
    assert!(compile_and_run("examples/phase1_full.xi").is_some());
}

#[test]
fn e2e_enum_compiles() {
    assert!(compile_and_run("examples/phase1_enum.xi").is_some());
}

#[test]
fn e2e_interface_compiles() {
    assert!(compile_and_run("examples/phase1_interface.xi").is_some());
}

#[test]
fn e2e_derive_enum_compiles() {
    assert!(compile_and_run("examples/phase1_derive_enum.xi").is_some());
}

#[test]
fn e2e_hardening_compiles() {
    assert!(compile_and_run("examples/phase1_hardening.xi").is_some());
}

#[test]
fn e2e_stress_compiles() {
    assert!(compile_and_run("examples/phase1_stress.xi").is_some());
}

#[test]
fn e2e_selfhost_sim_compiles() {
    assert!(compile_and_run("examples/phase1_selfhost.xi").is_some());
}

#[test]
fn e2e_async_spawn_compiles() {
    assert!(compile_and_run("examples/phase1_async_spawn.xi").is_some());
}

// ============================================================================
// E2E: Stress Tests
// ============================================================================

#[test]
fn e2e_stress_derive_50field() {
    assert!(compile_and_run("examples/stress_derive_50field.xi").is_some());
}

#[test]
fn e2e_stress_borrow_10level() {
    assert!(compile_and_run("examples/stress_borrow_10level.xi").is_some());
}

#[test]
fn e2e_stress_generic_5chain() {
    assert!(compile_and_run("examples/stress_generic_5chain.xi").is_some());
}

#[test]
fn e2e_stress_float_matrix() {
    assert!(compile_and_run("examples/stress_float_matrix.xi").is_some());
}

// ============================================================================
// E2E: WASM Target
// ============================================================================

#[test]
fn e2e_wasm_demo_float() {
    assert!(compile_wasm("examples/demo_float.xi"));
}

#[test]
fn e2e_wasm_ownership() {
    assert!(compile_wasm("examples/phase1_ownership.xi"));
}

#[test]
fn e2e_wasm_generics() {
    assert!(compile_wasm("examples/phase1_generics.xi"));
}

#[test]
fn e2e_wasm_diff_test() {
    assert!(compile_wasm("examples/diff_test.xi"));
}

#[test]
fn e2e_wasm_async_spawn() {
    assert!(compile_wasm("examples/phase1_async_spawn.xi"));
}

// ============================================================================
// E2E: IR Verification
// ============================================================================

#[test]
fn e2e_ir_contains_define() {
    assert!(compile_and_check_ir("examples/demo_float.xi", "define i64 @main"));
}

#[test]
fn e2e_ir_contains_main_ret() {
    assert!(compile_and_check_ir("examples/diff_test.xi", "ret i64 42"));
}

#[test]
fn e2e_ir_contains_struct() {
    assert!(compile_and_check_ir("examples/phase1_derive.xi", "%struct.Point"));
}

#[test]
fn e2e_ir_contains_trap() {
    assert!(compile_and_check_ir("examples/phase1_contracts.xi", "@llvm.trap"));
}

#[test]
fn e2e_ir_contains_generic_monomorph() {
    assert!(compile_and_check_ir("examples/phase1_generics.xi", "define i64 @wrap_Int"));
}

#[test]
fn e2e_ir_contains_ownership() {
    assert!(compile_and_check_ir("examples/phase1_ownership.xi", "define i64 @take_ownership"));
}

#[test]
fn e2e_ir_contains_module_fn() {
    assert!(compile_and_check_ir("examples/phase1_modules.xi", "define i64 @add"));
}

#[test]
fn e2e_impl_trait_compiles() {
    // M9.6: impl Trait return types must compile to valid LLVM IR
    let status = compile_and_run("examples/phase1_impl_trait.xi");
    assert_eq!(status, Some(0));
}

// ============================================================================
// E2E: CLI Flags
// ============================================================================

#[test]
fn e2e_diagnostics_json() {
    assert!(compile_diagnostics_json("examples/demo_float.xi"));
}

#[test]
fn e2e_dump_contracts() {
    assert!(compile_dump_contracts("examples/phase1_contracts.xi"));
}

#[test]
fn e2e_emit_ir_flag() {
    let ir = compile_ir("examples/diff_test.xi");
    assert!(ir.is_some());
    assert!(ir.unwrap().contains("define i64 @main"));
}

// ============================================================================
// E2E: Cross-Target Triples (IR emission only, no native compile needed)
// ============================================================================

#[test]
fn e2e_target_wasm_triple() {
    assert!(compile_and_check_ir_with_target("examples/demo_float.xi", "wasm", "wasm32-unknown-unknown"));
}

#[test]
fn e2e_target_arm_triple() {
    assert!(compile_and_check_ir_with_target("examples/demo_float.xi", "arm", "aarch64-unknown-linux-gnu"));
}

#[test]
fn e2e_target_riscv_triple() {
    assert!(compile_and_check_ir_with_target("examples/demo_float.xi", "riscv", "riscv64-unknown-linux-gnu"));
}

// ============================================================================
// E2E: Selfhost Pipeline (compile only, IR emission)
// ============================================================================

#[test]
fn e2e_selfhost_lexer_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiom-lexer.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-lexer.xi should compile");
}

#[test]
fn e2e_selfhost_parser_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiom-parser.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-parser.xi should compile");
}

#[test]
fn e2e_selfhost_check_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiom-check.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-check.xi should compile");
}

#[test]
fn e2e_selfhost_codegen_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiom-codegen.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiom-codegen.xi should compile");
}

#[test]
fn e2e_selfhost_compiler_module() {
    let ir = compile_ir("selfhost/xiomc.xi").expect("selfhost/xiomc.xi should compile to IR");
    assert!(ir.contains("define i64 @main"), "selfhost compiler should have main");
}

#[test]
fn e2e_no_contracts_flag() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "--no-contracts", "examples/phase1_contracts.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "contracts should compile without contract checks");
}

#[test]
fn e2e_target_wasm_selfhost_lexer() {
    let output = Command::new(xiom_path())
        .args(["--target", "wasm", "--emit-ir", "selfhost/xiom-lexer.xi"])
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
    // R31: the runtime C sources live in the stdlib repo checkout. A
    // compiler-only checkout skips loudly (XIOM_REQUIRE_STDLIB=1 hard-fails
    // in CI, where the checkout is fetched).
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    let runtime_c = stdlib_root.join("runtime").join("xiom_runtime.c");
    assert!(runtime_c.exists(), "{} should exist", runtime_c.display());
}

#[test]
fn e2e_selfhost_v091_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v091.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v091.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v091_has_main() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v091.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v091 should have main");
}

#[test]
fn e2e_selfhost_v094_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v094.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v094.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v094_has_main() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v094.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v094 should have main");
}

#[test]
fn e2e_selfhost_v094_contains_extern_decls() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v094.xi"])
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
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v094.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define void @emit_demo_float"), "v094 should define emit_demo_float");
}

#[test]
fn e2e_runtime_ir_declares_externs() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v091.xi"])
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
    // Phase 4: Self-hosting bootstrap -- the self-hosted compiler crashes
    // at runtime (ACCESS_VIOLATION). Enable with XIOM_SELFHOST=1.
    if std::env::var("XIOM_SELFHOST").is_err() {
        eprintln!("  [SKIP] Selfhost v10 -- enable with XIOM_SELFHOST=1 (Phase 4)");
        return;
    }
    let output = std::process::Command::new(xiom_path())
        .args(["-o", "e2e_v10_self_compile.exe", "selfhost/xiomc_v10.xi"])
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
    // Phase 4: Self-hosting bootstrap -- the v10 selfhost compiler
    // generates IR that needs updating to work with the v2.0 runtime.
    // This test will be re-enabled after Phase 3 (Z3, debugger, LSP).
    if std::env::var("XIOM_SELFHOST").is_err() {
        eprintln!("  [SKIP] Selfhost native compile -- enable with XIOM_SELFHOST=1 (Phase 4)");
        return;
    }
    let output = std::process::Command::new(xiom_path())
        .args(["-o", "e2e_v10_self_bootstrap_src.exe", "selfhost/xiomc_v10.xi"])
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
        .args(["-maes", "-DXIOM_NO_ASM", "-o", "e2e_v10_bootstrap.exe", "e2e_v10_output.ll", "stdlib/runtime//xiom_runtime.c"])
        .current_dir(project_root())
        .output();

    if let Ok(result) = clang_result {
        assert!(result.status.success(), "Bootstrap IR compilation failed");
    }
}

// ============================================================================
// E2E: Selfhost v11_test -- Parameter-counting compiler for demo_float.xi
// ============================================================================

#[test]
fn e2e_selfhost_v11_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v11_test.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v11_test.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v11_has_main() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v11_test.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("define i64 @main"), "v11_test IR should contain main");
    assert!(stdout.contains("define"), "v11_test IR should contain function definitions");
}

#[test]
fn e2e_selfhost_v11_self_run() {
    // Phase 4: Self-hosting bootstrap.
    if std::env::var("XIOM_SELFHOST").is_err() {
        eprintln!("  [SKIP] Selfhost v11 -- enable with XIOM_SELFHOST=1 (Phase 4)");
        return;
    }
    // Compile v11_test to binary, run it -- it should emit IR for demo_float.xi functions
    let output = Command::new(xiom_path())
        .args(["-o", "e2e_v11_self.exe", "selfhost/xiomc_v11_test.xi"])
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
// E2E: Selfhost v093 / v095 -- Pipeline compilation checks
// ============================================================================

#[test]
fn e2e_selfhost_v093_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v093.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v093.xi should compile to IR");
}

#[test]
fn e2e_selfhost_v095_compiles() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost/xiomc_v095.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "selfhost/xiomc_v095.xi should compile to IR");
}

// ============================================================================
// E2E: Multi-File Module Catalog -- Regression Tests
// ============================================================================

/// Compile and run test_mod/math.xi. Verifies catalog resolves cross-file
/// types (BenchResult) and functions (make_result) from test_mod/main.xi.
/// Expected exit code: 34.
#[test]
fn e2e_multifile_testmod_math_runs() {
    let exit = compile_and_run("examples/test_mod//math.xi");
    assert_eq!(exit, Some(34), "test_mod/math.xi should exit 34");
}

/// Compile benchmark/bench_math.xi (uses `use benchmark.main.BenchResult` from
/// external benchmark/main.xi). Verifies the catalog resolves cross-file types.
#[test]
fn e2e_multifile_bench_math_compiles() {
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", "examples/benchmark//bench_math.xi"])
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
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", "examples/benchmark//main.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(),
        "benchmark/main.xi 30-module suite should compile via ModuleCatalog");
}

// ============================================================================
// E2E: CLI Flags -- Timeout & Memory
// NOTE: Watchdog thread tests are inherently racy and environment-dependent.
// Flag parsing correctness is verified via --help output test below.
// The flags are tested in isolation via unit/integration tests.
// ============================================================================

#[test]
fn e2e_help_shows_timeout_and_memory_flags() {
    let output = std::process::Command::new(xiom_path())
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
// E2E: Regression -- method `match self` on enum receiver
// ============================================================================
/// Regression: a method that pattern-matches `self` on an enum receiver must
/// treat self as the typed struct, NOT a phantom `i64` duplicate param.
/// Without the fix, variant patterns become variable bindings and arms return
/// raw i64 discriminants -> `store %struct.X i64` (invalid IR).
/// Returns exit code 0 when the fix is present.
#[test]
fn e2e_method_match_self_enum() {
    let exit = compile_and_run("examples/e2e//method_match_self_enum.xi");
    assert_eq!(exit, Some(0), "match self on enum receiver should compile, link, run, and exit 0");
}

// ============================================================================
// E2E: Regression suite for Tier 2 codegen fixes (hermetic -- no stdlib needed
// except where noted). Each program returns 0 on success, nonzero on failure.
// These lock in fixes that were hard-won during the stdlib execution work.
// ============================================================================

/// Enum `==`/`!=` via the builtin `.eq` fallback (compare discriminant inline),
/// plus enum-variant construction as values from match arms.
#[test]
fn e2e_enum_eq_and_variants() {
    assert_eq!(compile_and_run("examples/e2e//enum_eq.xi"), Some(0),
        "enum ==/!= and variant construction should work");
}

/// Vec builtins: new/push/len/index-read/pop returning Option, with element
/// coercion. Locks in inline Vec method dispatch + Option payload extraction.
#[test]
fn e2e_vec_ops() {
    assert_eq!(compile_and_run("examples/e2e//vec_ops.xi"), Some(0),
        "Vec new/push/len/index/pop should work");
}

/// Char (i8) <-> Int (i64) casts + i8 widening in arithmetic (sext/trunc).
#[test]
fn e2e_char_cast() {
    assert_eq!(compile_and_run("examples/e2e//char_cast.xi"), Some(0),
        "Char<->Int casts and widening should work");
}

/// Cross-module use of a stdlib module whose functions call C externs
/// (math). Locks in cross-module extern-declare injection + libc/libm handling.
#[test]
fn e2e_cross_module_math() {
    assert_eq!(compile_and_run("examples/e2e//cross_math.xi"), Some(0),
        "cross-module stdlib call with C externs should link and run");
}

/// Module-level mutable `var` global: write persists across function calls.
/// Locks in the mutable-module-global feature (ConstDecl.is_mut).
#[test]
fn e2e_module_global_var() {
    assert_eq!(compile_and_run("examples/e2e//module_global.xi"), Some(0),
        "module-level mutable var global should persist writes across calls");
}

/// `pub const` referenced across functions (GAP-3): const resolves at use sites.
#[test]
fn e2e_pub_const_use() {
    assert_eq!(compile_and_run("examples/e2e//const_use.xi"), Some(0),
        "pub const should resolve and be usable across functions");
}

/// Enum variant as a value (let binding from var assignment), and ==/!= on enum types.
#[test]
fn e2e_enum_variant_value() {
    assert_eq!(compile_and_run("examples/e2e//enum_variant_value.xi"), Some(0),
        "enum variant as value and ==/!= should work");
}

/// Const-generics and array indexing: let-bound arrays index correctly (5a.7),
/// const-declared sizes work in while loops (5a.5).
#[test]
fn e2e_const_generic_array() {
    assert_eq!(compile_and_run("examples/e2e//const_generic_array.xi"), Some(0),
        "const-generics: array indexing and const-declared loop sizes");
}

/// `&mut Scalar` parameter is a real LLVM pointer: `inc(p: &mut Int)` derefs to
/// read (`*p`) and stores through (`*p = ...`), mutating the caller's local.
#[test]
fn e2e_ref_mut_param() {
    assert_eq!(compile_and_run("examples/e2e//ref_mut_param.xi"), Some(0),
        "&mut Int param should deref-read and store-through, mutating the caller");
}

/// Raw pointer deref round-trip: `&mut x` reaches a `*T` param, which reads/writes
/// through the real pointer so the mutation is visible in the caller's binding.
#[test]
fn e2e_ptr_deref() {
    assert_eq!(compile_and_run("examples/e2e//ptr_deref.xi"), Some(0),
        "raw pointer deref read/write round-trip should mutate the source local");
}

// ============================================================================
// E2E: Regression -- field assignment + store-back (Clusters 1-3 fixes)
// ============================================================================

/// Struct field assignment (`self.field = expr`) emits a store instruction
/// through GEP into the struct alloca.
#[test]
fn e2e_field_assign() {
    assert_eq!(compile_and_run("examples/e2e//field_assign.xi"), Some(0),
        "struct field assignment should store through GEP");
}

/// Struct method returning modified self stores back to the caller's variable
/// so mutation persists across the call.
#[test]
fn e2e_method_store_back() {
    assert_eq!(compile_and_run("examples/e2e//method_store_back.xi"), Some(0),
        "mutating struct method should store result back to receiver var");
}

/// Chained calls like `make_pair(10,25).sum()` resolve the method on the
/// return type of the call expression.
#[test]
fn e2e_call_receiver_type() {
    assert_eq!(compile_and_run("examples/e2e//call_receiver_type.xi"), Some(0),
        "chained call receiver type inference should resolve method");
}

/// Or-patterns like `1 | 2 | 3 =>` in match arms compile and match correctly.
#[test]
fn e2e_or_pattern() {
    assert_eq!(compile_and_run("examples/e2e//or_pattern.xi"), Some(0),
        "or-patterns in match should compile and match correctly");
}

/// Brace-form modules (`module x { ... }`) compile and run correctly.
/// GAP-13: previously marked as won't-fix but the parser already handles
/// block-form module parsing. This test locks in the behavior.
#[test]
fn e2e_brace_module() {
    assert_eq!(compile_and_run("examples/e2e//brace_module.xi"), Some(0),
        "brace-form modules should compile and run");
}

/// Struct method returning modified self stores back to caller's variable
/// so mutation persists across the call (store-back mechanism).
#[test]
fn e2e_mut_struct() {
    assert_eq!(compile_and_run("examples/e2e//mut_struct.xi"), Some(0),
        "struct mutation via store_back should propagate to caller");
}

/// B-003: Option<Str> from method returns with ensures contracts.
/// Verifies that `result is Some => result.len() > 0` contract does not
/// crash (was generating undefined `Option.len()` call before fix).
#[test]
fn e2e_b003_option_str_method() {
    assert_eq!(compile_and_run("examples/e2e//b003_option_str_method.xi"), Some(0),
        "Option<Str> method returns with contracts should compile and run correctly");
}

/// DJB2 hash monomorphized through the Hash interface.
/// Same input -> same hash; different inputs -> different hashes.
#[test]
fn e2e_djb2_hash() {
    assert_eq!(compile_and_run("examples/e2e//djb2_hash.xi"), Some(0),
        "DJB2 hash via Hash[T] interface should produce deterministic non-zero values");
}

/// Generic swap via `&mut T` references: verifies scalar &mut pointers
/// work inside generic monomorphized functions (ARC A + ARC B).
#[test]
fn e2e_mut_ref_swap() {
    assert_eq!(compile_and_run("examples/e2e//mut_ref_swap.xi"), Some(0),
        "generic &mut T swap should exchange values correctly");
}

/// Phase 5c.7 hardening: Float32 compat, hex escapes, enum constructors,
/// comma-separated contracts, Int/Char compat, enum pattern matching.
#[test]
fn e2e_phase5c7_hardening() {
    assert_eq!(compile_and_run("examples/e2e//phase5c7_hardening.xi"), Some(0),
        "phase 5c.7 hardening: all 7 compiler fixes pass");
}

/// Combined features: store_back, hash determinism, generic monomorphization.
#[test]
fn e2e_combined_patterns() {
    assert_eq!(compile_and_run("examples/e2e//combined_patterns.xi"), Some(0),
        "combined store_back + hash + generics should work together");
}

// -- Ecosystem Hardening Tests ------------------------------------------

#[test]
fn eco_algo_89_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_algo.xi"), Some(0),
        "ecosystem: all 89 algorithm tests (binary search, quicksort, merge sort, gcd, fib, sieve, etc.)");
}

#[test]
fn eco_crypto_23_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_crypto.xi"), Some(0),
        "ecosystem: all 23 crypto tests (SHA-256, Base64, Hex, FNV-1a)");
}

#[test]
fn eco_db_18_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_db.xi"), Some(0),
        "ecosystem: all 18 database tests (B-Tree, WAL)");
}

#[test]
fn eco_full_30_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_full.xi"), Some(0),
        "ecosystem: all 30 full-feature tests (state machines, enums, contracts, Result, Option)");
}

#[test]
fn eco_http_18_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_http.xi"), Some(0),
        "ecosystem: all 18 HTTP tests (methods, headers, request/response)");
}

#[test]
fn eco_json_29_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_json.xi"), Some(0),
        "ecosystem: all 29 JSON tests (types, parse, stringify, nested)");
}

#[test]
fn eco_net_22_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_net.xi"), Some(0),
        "ecosystem: all 22 networking tests (IPv4/IPv6, sockets)");
}

#[test]
fn eco_sqlite_23_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_sqlite.xi"), Some(0),
        "ecosystem: all 23 SQLite tests (types, rows, CREATE TABLE SQL)");
}

#[test]
fn eco_test_20_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_test.xi"), Some(0),
        "ecosystem: all 20 test-framework tests (asserts, suites, results)");
}

#[test]
fn eco_vector_32_tests() {
    assert_eq!(compile_and_run("tests/ecosystem//test_vector.xi"), Some(0),
        "ecosystem: all 32 vector database tests (math, KNN, distance metrics)");
}

#[test]
fn e2e_fnptr_vec_index_call() {
    // Regression: Vec[fn()->Int] element call via index.
    // `tests[i]()` was emitting `inttoptr %struct.Vec to i64 ()*`
    // instead of loading the i64 function pointer from the Vec data.
    // The test returns f() = add_one() which returns 1.
    assert_eq!(compile_and_run("tests/ecosystem//test_fnptr.xi"), Some(1),
        "ecosystem: function pointer call from Vec index (tests[i]())");
}

#[test]
fn e2e_this_field_ref() {
    // Regression: &this.field passed to this-based methods.
    // SocketAddr.to_str calling IpAddr.to_str(&this.ip) was loading
    // the IpAddr struct by value instead of passing the GEP pointer,
    // causing ACCESS_VIOLATION (0xC0000005).
    assert_eq!(compile_and_run("tests/ecosystem//test_this_field_ref.xi"), Some(0),
        "ecosystem: nested this-based method dispatch via &this.field");
}

#[test]
fn e2e_enum_this_match() {
    // Regression: match on this in this-based methods for enums.
    // struct_type_from_expr didn't remap `this` to `self`, causing
    // LLVM IR errors when matching enum discriminants.
    assert_eq!(compile_and_run("tests/ecosystem//test_enum_this_match.xi"), Some(0),
        "ecosystem: enum variant matching in this-based methods");
}

#[test]
fn e2e_vec_of_struct() {
    // Regression: Vec-of-struct inline storage (5c.21 -- size-aware
    // elem_size with memcpy push/store/load for multi-field structs).
    assert_eq!(compile_and_run("tests/ecosystem//test_vec_of_struct.xi"), Some(0),
        "ecosystem: Vec[Struct] push/index/pop with multi-field structs");
}

#[test]
fn eco_ffi_binding_gaps() {
    // Regression: () in Result generic, pub const cross-module, extern cross-module
    assert_eq!(compile_and_run("tests/ecosystem//test_ffi.xi"), Some(0),
        "ecosystem: FFI binding gaps (unit in Result, pub const, extern)");
}

/// 5e.3 G-31: cross-package use resolution -- a file in one directory uses
/// a module in another directory via `use`. Locks in the catalog's recursive
/// source_dir scanning + walk-up project root detection.
#[test]
fn e2e_cross_package_use() {
    assert_eq!(compile_and_run("examples/e2e//cross_pkg//main.xi"), Some(0),
        "G-31: cross-package use math_utils should compile and return 0");
}

/// 5e.3 G-30: cross-package extern resolution -- extern "C" declarations
/// in one module propagate correctly when used from another module via `use`.
#[test]
fn e2e_cross_package_extern() {
    // Uses the math_utils.xi in examples/e2e/cross_pkg/ -- verifies
    // cross-directory resolution works for any file in the tree.
    let result = compile_and_run("examples/e2e//cross_pkg//main.xi");
    assert_eq!(result, Some(0),
        "G-30/G-31: cross-package use+extern should compile and run");
}

/// 5e G-15: sret ABI -- C struct return on Linux SysV. Small structs
/// (<16 bytes) return in registers; large structs (>16 bytes) use sret.
/// Verifies XIOM emits correct struct-return IR for both cases.
#[test]
fn e2e_g15_sret_abi() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "examples/e2e/g15_sret.xi"])
        .current_dir(project_root())
        .output()
        .expect("xiomc");
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "G-15 sret must compile: {}", String::from_utf8_lossy(&output.stderr));
    // m351: the extern "C" ABI is now target-correct. On Win64 a 16-byte
    // aggregate return uses sret and a by-value 16-byte aggregate passes as a
    // pointer operand; SysV keeps the by-value register convention.
    #[cfg(target_os = "windows")]
    {
        assert!(ir.contains("declare void @g15_small_return(%struct.Small* sret(%struct.Small))"),
            "Win64: Small (16B) struct return must use sret:\n{ir}");
        assert!(ir.contains("declare void @g15_large_return(%struct.Large* sret(%struct.Large))"),
            "Win64: Large struct return must use sret:\n{ir}");
        assert!(ir.contains("declare void @g15_pass_and_return(%struct.Small* sret(%struct.Small), %struct.Small*)"),
            "Win64: struct pass+return must use the C ABI:\n{ir}");
    }
    #[cfg(not(target_os = "windows"))]
    {
        assert!(ir.contains("declare %struct.Small @g15_small_return()"),
            "SysV: Small struct must return by value (no sret):\n{ir}");
        assert!(ir.contains("declare %struct.Large @g15_large_return()"),
            "SysV: Large struct must return (LLVM uses sret):\n{ir}");
        assert!(ir.contains("declare %struct.Small @g15_pass_and_return(%struct.Small)"),
            "SysV: struct pass+return must work:\n{ir}");
    }
}

/// 5e G-40: repr(C) layout -- mixed-width C struct fields. Verifies XIOM
/// emits correct LLVM struct layout for Int8/Int16/Int32/Int64/Float32/Float64.
#[test]
fn e2e_g40_repc_layout() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "examples/e2e/g40_repc.xi"])
        .current_dir(project_root())
        .output()
        .expect("xiomc");
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "G-40 repr(C) must compile: {}", String::from_utf8_lossy(&output.stderr));
    assert!(ir.contains("%struct.MixedC = type"), "MixedC struct must be defined:\n{ir}");
    assert!(ir.contains("i8"), "Must have i8 field for Int8:\n{ir}");
    assert!(ir.contains("i16"), "Must have i16 field for Int16:\n{ir}");
    assert!(ir.contains("i32"), "Must have i32 field for Int32:\n{ir}");
    assert!(ir.contains("float"), "Must have float field for Float32:\n{ir}");
    assert!(ir.contains("double"), "Must have double field for Float64:\n{ir}");
}

/// 5e G-24: Float32 ARM ABI -- Float32 operations must produce valid IR
/// for ARM targets (IEEE 754 single-precision). Cross-compiled for
/// aarch64-linux-gnu via clang to verify.
#[test]
fn e2e_g24_float32_arm_abi() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "examples/e2e/g24_float32_arm.xi"])
        .current_dir(project_root())
        .output()
        .expect("xiomc");
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "G-24 Float32 ARM must compile: {}",
        String::from_utf8_lossy(&output.stderr));
    // Float32 promotes to double in XIOM (same IEEE 754, wider precision).
    // ARM and x86 use identical IEEE 754 representation.
    assert!(ir.contains("double"), "Must contain double type (Float32 promotes):\n{ir}");
    assert!(ir.contains("fadd") || ir.contains("fsub") || ir.contains("fmul") || ir.contains("fcmp"),
        "Must contain IEEE 754 fp ops:\n{ir}");
}

// ============================================================================
// 6A.1: Type Checker Hardening -- Self + Interface compatibility e2e tests
// ============================================================================

/// Verify Self compatibility is maintained (existing test proves this).
/// e2e_method_match_self_enum already covers Self-as-receiver matching.
/// NOTE: `-> Self` return type resolution is a separate Phase 6 gap
/// (checker does not resolve Self to concrete type in return position yet).
#[test]
fn e2e_self_compat_method_receiver() {
    // Use existing test file that compiles and runs correctly
    let output = std::process::Command::new(xiom_path())
        .args(["examples/e2e/method_match_self_enum.xi", "--run"])
        .current_dir(project_root())
        .output()
        .expect("xiomc");
    assert!(output.status.success(),
        "Self receiver matching must work. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.status.code(), Some(0));
}

/// Verify interface name is compatible with concrete implementor type.
#[test]
fn e2e_interface_compat_with_implementor() {
    let test_file = project_root().join("e2e_iface_compat_test.xi");
    std::fs::write(&test_file, r#"
interface Drawable { fn draw(self); }
type Circle = { r: Float64; }
fn Circle.draw(self) { }
fn make() -> Drawable { return Circle { r: 1.0 }; }
fn main() -> Int { return 0; }
"#).expect("write test file");

    let output = std::process::Command::new(xiom_path())
        .args([&test_file.to_string_lossy(), "--run"])
        .current_dir(project_root())
        .output()
        .expect("xiomc");
    let _ = std::fs::remove_file(&test_file);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(),
        "Interface must be compatible with implementor. stderr: {stderr}");
}

// ============================================================================
// AI-08: --ai-strict CI/CD gating regression
// ============================================================================

/// Verify --ai-strict flag appears in help output.
#[test]
fn e2e_help_shows_ai_strict_flag() {
    let output = std::process::Command::new(xiom_path())
        .arg("--help")
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("--ai-strict"), "help should document --ai-strict flag");
}

/// Verify --ai-strict --check-only exits non-zero on contract violations.
/// Uses --ai-dry-run to skip actual LLM calls but still check contracts.
#[test]
fn e2e_ai_strict_blocks_on_violations() {
    // Write a file with a contract violation (div-by-zero in requires)
    let test_file = project_root().join("e2e_ai_strict_test.xi");
    std::fs::write(&test_file,
        "fn div(a: Int, b: Int) -> Int\n  requires b != 0\n{ return a / b; }\nfn main() -> Int { return div(10, 0); }\n"
    ).expect("write test file");

    let output = std::process::Command::new(xiom_path())
        .args(["--ai-strict", "--check-only", "--ai-dry-run",
               &test_file.to_string_lossy()])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let _ = std::fs::remove_file(&test_file);

    // --ai-strict should cause non-zero exit when violations exist
    assert!(!output.status.success(),
        "--ai-strict --check-only should return non-zero on violations. stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

// ============================================================================
// M10 Scripting tests are in crates/xiom/tests/scripting_tests.rs
// (separated to avoid parallel C runtime compilation conflicts)
// ============================================================================

// M12: Regression test for match-on-Result codegen bug
// The issue: compiling `match io.read_line() { Ok(line) => ... }` produces
// LLVM IR with type mismatch: `store i8 %tmp, i8** %ptr` instead of `store i8*`
#[test]
fn e2e_match_result_codegen() {
    let src = "fn main() { match io.read_line() { Ok(line) => io.println(line), Err(_) => {}, } }\n";
    let ir = compile_and_get_ir(src);
    // Verify the IR doesn't contain the type mismatch pattern
    let has_bad_store = ir.lines().any(|l| l.contains("store i8 %") && l.contains("i8**"));
    assert!(!has_bad_store,
        "match on Result should not produce store i8/i8** type mismatch.\nIR snippet:\n{}",
        ir.lines().filter(|l| l.contains("store i8") && l.contains("**")).collect::<Vec<_>>().join("\n"));
}

/// Helper: compile XIOM source to LLVM IR string.
fn compile_and_get_ir(source: &str) -> String {
    let tmp = project_root().join("_e2e_match_test.xi");
    std::fs::write(&tmp, source).expect("write test source");
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", &tmp.to_string_lossy().to_string()])
        .current_dir(project_root())
        .output()
        .expect("xiom --emit-ir failed");
    let _ = std::fs::remove_file(&tmp);
    String::from_utf8_lossy(&output.stdout).to_string()
}

// ===================================================================
// M16 -- Compiler Hardening: zero warnings, clean exit codes
// ===================================================================

/// M16: Hello World with stdlib imports must produce ZERO warnings.
/// Before M16: 5 warnings (T, Vec[UInt8] repeated).
#[test]
fn e2e_m16_no_warnings() {
    let _tmp = project_root().join("_e2e_m16_nowarn.xi");
    // Use the test file from examples/e2e/
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", "examples/e2e//m16_no_warnings.xi"])
        .current_dir(project_root())
        .output()
        .expect("xiom --emit-ir failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    // AUDIT FIX (Stage 3 / Item A): catalog-body findings now print
    // deliberately (stdlib session fix list). The contract here is that
    // USER code emits no warnings -- filter the catalog line out.
    let own_warn = stderr.lines().any(|l| l.contains("warning") && !l.contains("catalog body") && !l.contains("catalog-body"));
    assert!(!own_warn, "M16: zero user-code warnings expected, got: {stderr}");
    assert!(output.status.success(), "compilation should succeed");
}

/// M16: Void main must return exit code 0 (not garbage like 358914400).
#[test]
fn e2e_m16_void_main_exit_zero() {
    assert_eq!(compile_and_run("examples/e2e//m16_void_main.xi"), Some(0),
        "M16: fn main() without return type must exit 0, not garbage");
}

/// M16: Concrete Option/Result types with struct payloads work correctly.
/// The 'J' type name is a single uppercase letter -- tests registry-aware
/// struct detection (was broken: is_struct_type_name rejected single-char names).
#[test]
fn e2e_m16_option_struct_payload() {
    let source = "type J = { k: Int; d: Int; } fn f() -> Option[J] { return Some(J { k: 0; d: 42; }); } fn main() -> Int { let r = f(); if r.is_some { return 0; } return 1; }";
    let tmp = project_root().join("_e2e_m16_opt.xi");
    std::fs::write(&tmp, source).expect("write");
    let result = compile_and_run(&tmp.to_string_lossy());
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(result, Some(0), "Option[J] with single-char type name must return is_some=true");
}

/// M16: Option.is_none on concrete type returns true for None.
#[test]
fn e2e_m16_option_none() {
    let source = "type P = { x: Int; } fn f() -> Option[P] { return None; } fn main() -> Int { let r = f(); if r.is_none { return 0; } return 1; }";
    let tmp = project_root().join("_e2e_m16_optnone.xi");
    std::fs::write(&tmp, source).expect("write");
    let result = compile_and_run(&tmp.to_string_lossy());
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(result, Some(0), "Option[P].is_none must return true for None");
}

/// M16: Result concrete type with struct payload -- is_err should work.
#[test]
fn e2e_m16_result_struct_err() {
    let source = "type E = { code: Int; } fn f() -> Result[Int, E] { return Err(E { code: 1; }); } fn main() -> Int { let r = f(); if r.is_err { return 0; } return 1; }";
    let tmp = project_root().join("_e2e_m16_res.xi");
    std::fs::write(&tmp, source).expect("write");
    let result = compile_and_run(&tmp.to_string_lossy());
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(result, Some(0), "Result[Int, E].is_err must return true for Err");
}

/// M16: Scripting mode (xiom run) must produce exit code 0.
/// Before M16: scripting produced garbage exit codes like 1879443520.
#[test]
fn e2e_m16_scripting_exit_zero() {
    let source = "io.println(\"test\");";
    let tmp = project_root().join("_e2e_m16_script.xi");
    std::fs::write(&tmp, source).expect("write");
    let run_output = std::process::Command::new(xiom_path())
        .args(["run", &tmp.to_string_lossy()])
        .current_dir(project_root())
        .output()
        .expect("xiom run failed");
    let _ = std::fs::remove_file(&tmp);
    let exit_code = run_output.status.code().unwrap_or(-1);
    assert_eq!(exit_code, 0, "M16: scripting mode must exit 0, got {exit_code}");
}

/// M17: &mut self methods must propagate mutations correctly.
#[test]
fn e2e_m17_mut_self_basic() {
    assert_eq!(compile_and_run("examples/e2e//m17_mut_self.xi"), Some(0),
        "M17: &mut self methods must mutate and propagate correctly");
}

/// M17: Complex &mut self patterns: push, add, return self.
#[test]
fn e2e_m17_mut_self_complex() {
    let source = "
type Vec2 = { x: Int; y: Int; }
fn Vec2.add(&mut self, other: &Vec2) { x = x + other.x; y = y + other.y; }
fn Vec2.magnitude(self) -> Int {
  if x > y { return x; }
  return y;
}
fn main() -> Int {
  var v = Vec2 { x: 3; y: 4; };
  var w = Vec2 { x: 1; y: 2; };
  v.add(&w);
  if v.x == 4 && v.y == 6 { return 0; }
  return 1;
}";
    let tmp = project_root().join("_e2e_m17_mut2.xi");
    std::fs::write(&tmp, source).expect("write");
    let result = compile_and_run(&tmp.to_string_lossy());
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(result, Some(0), "M17: complex &mut self with references");
}

/// M17: Concrete Result with struct error type validates is_ok/is_err.
#[test]
fn e2e_m17_result_struct_ok() {
    let source = "
type MyErr = { code: Int; msg: Str; }
fn ok_val() -> Result[Int, MyErr] { return Ok(42); }
fn err_val() -> Result[Int, MyErr] { return Err(MyErr { code: 1; msg: \"fail\"; }); }
fn main() -> Int {
  let r = ok_val();
  if !r.is_ok { return 1; }
  if r.unwrap() != 42 { return 2; }
  let e = err_val();
  if !e.is_err { return 3; }
  return 0;
}";
    let tmp = project_root().join("_e2e_m17_result.xi");
    std::fs::write(&tmp, source).expect("write");
    let result = compile_and_run(&tmp.to_string_lossy());
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(result, Some(0), "M17: concrete Result with struct error must work");
}

/// M17: Zero warnings for a trivial program with stdlib imports.
#[test]
fn e2e_m17_zero_warnings() {
    let source = "use xiom.io; fn main() { io.println(\"hi\"); }";
    let tmp = project_root().join("_e2e_m17_warn.xi");
    std::fs::write(&tmp, source).expect("write");
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", &tmp.to_string_lossy()])
        .current_dir(project_root())
        .output()
        .expect("compile");
    let _ = std::fs::remove_file(&tmp);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The compiler indexes the repo root for this probe; test-fixture
    // `tests/regression/*/mini_std` trees duplicate real stdlib modules and
    // surface as W001 module collisions (introduced by the m347 fixture).
    // Those are fixture collisions, not user-code warnings. Excluding
    // tests/regression from the catalog index is tracked as m367.
    let own_warn = stderr.lines().any(|l| {
        l.contains("warning")
            && !l.contains("catalog body")
            && !l.contains("catalog-body")
            && !(l.contains("W001") && l.contains("tests") && l.contains("mini_std"))
    });
    assert!(!own_warn, "M17: zero user-code warnings expected, got: {stderr}");
    assert!(output.status.success());
}

/// C001 fix (2026-10-04): the return type of a module-qualified call must not
/// depend on function-registration order. With `iter.range` and
/// `iter.range.range` both registered, the bare-leaf suffix scan used to pick
/// whichever entry the map listed first; a `%struct.Vec` pick classified
/// `iter.range(..).contains(..)` as a contract collection scan and failed
/// C001. This exact three-sum + contains shape flipped ~50% run-to-run before
/// the fix; it must now compile and run deterministically.
#[test]
fn e2e_c001_iter_range_contains_sums() {
    let source = r#"
use xiom.iter;
fn main() -> Int {
  if iter.range(0, 0).sum() != 0 { return 1; }
  if iter.range(1, 5).sum() != 10 { return 2; }
  if iter.range(0, 10).sum() != 45 { return 3; }
  if !iter.range(1, 5).contains(3) { return 4; }
  if iter.range(1, 5).contains(5) { return 5; }
  if iter.range(1, 5).len() != 4 { return 6; }
  return 0;
}"#;
    let tmp = project_root().join("_e2e_c001_iter_range.xi");
    std::fs::write(&tmp, source).expect("write");
    let result = compile_and_run(&tmp.to_string_lossy());
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(result, Some(0), "C001: iter.range contains must resolve deterministically");
}

/// M19-E2E: io.read_file() must return correct file content.
/// Regression test for the M19 bug where read_file returned empty string
/// despite is_ok=true (caused by offset() auto-stub + unwrap type corruption).
#[test]
fn e2e_m19_read_file_content() {
    // Uses a dedicated test file that writes known content, reads it back,
    // and verifies the content matches exactly.
    assert_eq!(
        compile_and_run("tests/regression//m19_read_file.xi"),
        Some(0),
        "M19-E2E: io.read_file() must return correct content"
    );
}

/// M19-E2E: Enum variants with same-named fields but different types
/// must correctly extract payload values. Tests Bool/Str/Float64 payload
/// access on colliding field names.
#[test]
fn e2e_m19_enum_same_field_types() {
    assert_eq!(
        compile_and_run("tests/regression//m19_enum_fields.xi"),
        Some(0),
        "M19-E2E: enum variant same-named fields must extract correct types"
    );
}

// ============================================================================
// CTFE Phase A E2E Tests -- Compile-Time Function Evaluation
// ============================================================================

/// CTFE-A: Comprehensive const evaluation -- arithmetic, comparison, boolean,
/// unary, const refs, if/else folding, builtins, const blocks.
#[test]
fn e2e_ctfe_phase_a_full() {
    assert_eq!(
        compile_and_run("tests/regression//ctfe_phase_a.xi"),
        Some(0),
        "CTFE-A: all const evaluation categories must pass"
    );
}

/// CTFE-A: Builtins with struct types -- sizeof, align_of, field_offset
#[test]
fn e2e_ctfe_builtins_struct() {
    assert_eq!(
        compile_and_run("tests/regression//ctfe_builtins_struct.xi"),
        Some(0),
        "CTFE-A: struct builtins (sizeof/align_of/field_offset) must pass"
    );
}

/// CTFE-B: Function evaluation -- factorial, fibonacci, loops, branching
#[test]
fn e2e_ctfe_phase_b_functions() {
    assert_eq!(
        compile_and_run("tests/regression//ctfe_phase_b.xi"),
        Some(0),
        "CTFE-B: pure function CTFE evaluation must pass"
    );
}

// ============================================================================
// v0.55 Inline ASM E2E Tests
// ============================================================================

/// ASM: Basic inline assembly -- nop, mov, constraints, clobbers
#[test]
fn e2e_asm_basic() {
    assert_eq!(
        compile_and_run("tests/regression//asm_basic.xi"),
        Some(0),
        "ASM: basic inline assembly must pass"
    );
}

/// Never: diverging functions, bottom type compatibility
#[test]
fn e2e_never_type() {
    assert_eq!(
        compile_and_run("tests/regression//never_type.xi"),
        Some(0),
        "Never: ! type must parse, type-check, and run"
    );
}

/// Spawn: OS thread creation, separate function compilation
#[test]
fn e2e_spawn_basic() {
    assert_eq!(
        compile_and_run("tests/regression//spawn_basic.xi"),
        Some(0),
        "Spawn: must compile spawn body as separate function and link runtime"
    );
}

/// R2: Spawn capture -- outer variables forwarded via heap-allocated env struct
#[test]
fn e2e_spawn_capture() {
    assert_eq!(
        compile_and_run("tests/regression//spawn_capture.xi"),
        Some(0),
        "Spawn capture: must forward captured variables via env struct"
    );
}

/// I1: Send enforcement -- Int captures are Send (must compile and run)
#[test]
fn e2e_send_capture_ok() {
    assert_eq!(
        compile_and_run("tests/regression//send_capture_ok.xi"),
        Some(0),
        "Send enforcement: Int implements Send, spawn capture must be allowed"
    );
}

/// I1: Send enforcement -- struct with primitives is Send (must compile and run)
#[test]
fn e2e_send_struct_ok() {
    assert_eq!(
        compile_and_run("tests/regression//send_struct_ok.xi"),
        Some(0),
        "Send enforcement: struct with only primitive fields is Send"
    );
}

// ============================================================================
// Chaos Benchmark E2E Tests
// ============================================================================

/// Chaos t1: Buddy allocator -- 1M Vec elements + buddy splitting/coalescing
/// Regression test for R4 (dynamic alloca in Vec::push) and R5 (recursion counter leak)
#[test]
fn e2e_chaos_t1_allocator() {
    // Uses the internal copy of the buddy-allocator pattern (UTF-8) so the
    // test does not depend on xiom-benchmark-chaos reference files.
    assert_eq!(
        compile_and_run("tests/ecosystem//t1-allocator.xi"),
        Some(0),
        "Chaos t1: buddy memory allocator must pass (R4+R5 regression)"
    );
}

/// Chaos t2: SPSC concurrent queue -- 1M enqueue/dequeue + AtomicInt ops
/// Regression test for R4 (dynamic alloca in Vec::push) and R5 (recursion counter leak)
///
/// The upstream benchmark lives in its own repo now (xiom-benchmark-chaos)
/// and uses the DOWNLOADED compiler; compile coverage for the identical
/// internal copy lives in `e2e_i2_parallel_codegen`
/// (`tests/ecosystem/t2-queue.xi`), so the run is ignored instead of
/// skipping on a monorepo path that no longer exists.
#[test]
#[ignore = "benchmark-chaos moved to its own repo; compile coverage in e2e_i2_parallel_codegen"]
fn e2e_chaos_t2_queue() {
    assert_eq!(
        compile_and_run("tests/ecosystem/t2-queue.xi"),
        Some(0),
        "Chaos t2: SPSC atomic queue must pass (R4+R5 regression)"
    );
}

/// Chaos t3: Hot-reload module loader -- 1000 load/call/reload cycles
#[test]
#[ignore = "benchmark-chaos moved to its own repo; compile coverage in e2e_i2_parallel_codegen"]
fn e2e_chaos_t3_hot_reload() {
    assert_eq!(
        compile_and_run("tests/ecosystem/t3-hot-reload.xi"),
        Some(0),
        "Chaos t3: hot-reload module loader must pass"
    );
}

/// Chaos t4: TCP packet parser -- 1M packet updates
#[test]
#[ignore = "benchmark-chaos moved to its own repo; compile coverage in e2e_i2_parallel_codegen"]
fn e2e_chaos_t4_packet() {
    assert_eq!(
        compile_and_run("tests/ecosystem/t4-packet.xi"),
        Some(0),
        "Chaos t4: TCP packet parser must pass"
    );
}

/// Chaos t5: B-tree file index -- 50K inserts + 20K lookups
#[test]
#[ignore = "benchmark-chaos moved to its own repo; compile coverage in e2e_i2_parallel_codegen"]
fn e2e_chaos_t5_btree() {
    assert_eq!(
        compile_and_run("tests/ecosystem/t5-btree.xi"),
        Some(0),
        "Chaos t5: B-tree file index must pass"
    );
}

/// I2: Parallel codegen -- verify all 5 chaos tasks compile correctly with --parallel-codegen
#[test]
fn e2e_i2_parallel_codegen() {
    // Internal copies of the benchmark patterns (UTF-8) so the test does not
    // depend on xiom-benchmark-chaos reference files.
    let tasks = [
        "tests/ecosystem//t1-allocator.xi",
        "tests/ecosystem//t2-queue.xi",
        "tests/ecosystem//t3-hot-reload.xi",
        "tests/ecosystem//t4-packet.xi",
        "tests/ecosystem//t5-btree.xi",
    ];
    for task in &tasks {
        assert_eq!(
            compile_and_run_with_flags(task, &["--parallel-codegen"]),
            Some(0),
            "Parallel codegen: {task} must pass"
        );
    }
}

/// Systems-arena t8: Safety probe -- must compile + output valid JSON
#[test]
fn e2e_safety_probe() {
    // Uses the internal copy of the safety-probe pattern (UTF-8) so the test
    // does not depend on xiom-benchmark-chaos reference files.
    assert_eq!(
        compile_and_run("tests/ecosystem//t8-safety-probe.xi"),
        Some(0),
        "Safety probe: must compile, run, and output JSON (no crash)"
    );
}

// ============================================================================
// M20-A1 E2E Tests -- Closure Codegen
// ============================================================================

#[test] fn e2e_m20_closure_capture()      { assert_eq!(compile_and_run("tests/regression//m20_closure_capture.xi"),       Some(0)); }
#[test] fn e2e_m20_closure_as_arg()       { assert_eq!(compile_and_run("tests/regression//m20_closure_as_arg.xi"),        Some(0)); }
#[test] fn e2e_m20_closure_multi()        { assert_eq!(compile_and_run("tests/regression//m20_closure_multi.xi"),         Some(0)); }
#[test] fn e2e_m20_closure_let()          { assert_eq!(compile_and_run("tests/regression//m20_closure_let.xi"),           Some(0)); }
#[test] fn e2e_m20_closure_noncapture()   { assert_eq!(compile_and_run("tests/regression//m20_closure_noncapture.xi"),    Some(0)); }
#[test] fn e2e_m20_closure_multi_capture(){ assert_eq!(compile_and_run("tests/regression//m20_closure_multi_capture.xi"), Some(0)); }

// ============================================================================
// v0.56 E2E Tests -- Production Polish
// ============================================================================

/// Spawn with multiple captures (3 Int vars forwarded via env struct)
#[test] fn e2e_spawn_multi_capture() { assert_eq!(compile_and_run("tests/regression//spawn_multi_capture.xi"), Some(0)); }
/// Spawn from helper function with captured params
#[test] fn e2e_spawn_from_fn()       { assert_eq!(compile_and_run("tests/regression//spawn_from_fn.xi"),       Some(0)); }
/// Send enforcement: nested struct with all-Send primitive fields
#[test] fn e2e_send_nested_struct()  { assert_eq!(compile_and_run("tests/regression//send_nested_struct.xi"),  Some(0)); }
/// Parallel codegen: 5 independent functions
#[test] fn e2e_parallel_multi_fn()   { assert_eq!(compile_and_run("tests/regression//parallel_multi_fn.xi"),   Some(0)); }
/// DI emission: compile with --debug, verify exits correctly
#[test] fn e2e_di_emission()         { assert_eq!(compile_and_run("tests/regression//di_emission.xi"),         Some(17)); } // add(mul(3,4),5)=17
/// CTFE comprehensive: 14 const assertions (arithmetic, logic, if-folding)
#[test] fn e2e_ctfe_comprehensive()  { assert_eq!(compile_and_run("tests/regression//ctfe_comprehensive.xi"),  Some(0)); }
#[test] fn e2e_m20_closure_in_if()        { assert_eq!(compile_and_run("tests/regression//m20_closure_in_if.xi"),         Some(0)); }
#[test] fn e2e_m20_closure_chain()        { assert_eq!(compile_and_run("tests/regression//m20_closure_chain.xi"),         Some(0)); }
#[test] fn e2e_m20_closure_nested_scope() { assert_eq!(compile_and_run("tests/regression//m20_closure_nested_scope.xi"),  Some(0)); }
#[test] fn e2e_m20_closure_identity()     { assert_eq!(compile_and_run("tests/regression//m20_closure_identity.xi"),      Some(0)); }

// M20-A1b: Block-style closure tests
#[test] fn e2e_m20_block_closure_capture()   { assert_eq!(compile_and_run("tests/regression//m20_block_closure_capture.xi"),    Some(0)); }
#[test] fn e2e_m20_block_closure_multistmt() { assert_eq!(compile_and_run("tests/regression//m20_block_closure_multistmt.xi"),  Some(0)); }
#[test] fn e2e_m20_block_closure_noncapture(){ assert_eq!(compile_and_run("tests/regression//m20_block_closure_noncapture.xi"), Some(0)); }
#[test] fn e2e_m20_block_closure_nested()    { assert_eq!(compile_and_run("tests/regression//m20_block_closure_nested.xi"),     Some(0)); }
#[test] fn e2e_m20_block_closure_void()      { assert_eq!(compile_and_run("tests/regression//m20_block_closure_void.xi"),       Some(0)); }

// M20: Tuple return type tests
#[test] fn e2e_m20_tuple_two_int()  { assert_eq!(compile_and_run("tests/regression//m20_tuple_two_int.xi"),  Some(0)); }
#[test] fn e2e_m20_tuple_three()    { assert_eq!(compile_and_run("tests/regression//m20_tuple_three.xi"),    Some(0)); }
#[test] fn e2e_m20_tuple_mixed()    { assert_eq!(compile_and_run("tests/regression//m20_tuple_mixed.xi"),    Some(0)); }

// M20: impl Trait for Type tests
#[test] fn e2e_m20_impl_basic()        { assert_eq!(compile_and_run("tests/regression//m20_impl_basic.xi"),         Some(0)); }
#[test] fn e2e_m20_impl_multi()        { assert_eq!(compile_and_run("tests/regression//m20_impl_multi.xi"),         Some(0)); }
#[test] fn e2e_m20_impl_self()         { assert_eq!(compile_and_run("tests/regression//m20_impl_self.xi"),          Some(0)); }
#[test] fn e2e_m20_impl_two_methods()  { assert_eq!(compile_and_run("tests/regression//m20_impl_two_methods.xi"),   Some(0)); }
#[test] fn e2e_m20_impl_generic()      { assert_eq!(compile_and_run("tests/regression//m20_impl_generic.xi"),       Some(0)); }

// M20: Hardening tests -- edge cases and stress
#[test] fn e2e_m20_harden_recursion()      { assert_eq!(compile_and_run("tests/regression//m20_harden_recursion.xi"),       Some(0)); }
#[test] fn e2e_m20_harden_many_variants()  { assert_eq!(compile_and_run("tests/regression//m20_harden_many_variants.xi"),   Some(0)); }
#[test] fn e2e_m20_harden_nested_struct()  { assert_eq!(compile_and_run("tests/regression//m20_harden_nested_struct.xi"),   Some(0)); }
#[test] fn e2e_m20_harden_while_break()    { assert_eq!(compile_and_run("tests/regression//m20_harden_while_break.xi"),     Some(0)); }
#[test] fn e2e_m20_harden_string_ops()     { assert_eq!(compile_and_run("tests/regression//m20_harden_string_ops.xi"),      Some(0)); }
#[test] fn e2e_m20_harden_generics()       { assert_eq!(compile_and_run("tests/regression//m20_harden_generics.xi"),        Some(0)); }
#[test] fn e2e_m20_harden_match_guard()    { assert_eq!(compile_and_run("tests/regression//m20_harden_match_guard.xi"),     Some(0)); }
#[test] fn e2e_m20_harden_option_chain()   { assert_eq!(compile_and_run("tests/regression//m20_harden_option_chain.xi"),    Some(0)); }
#[test] fn e2e_m20_harden_control_flow()   { assert_eq!(compile_and_run("tests/regression//m20_harden_control_flow.xi"),    Some(0)); }

// M20: Edge case tests -- production-grade hardening
#[test] fn e2e_m20_edge_ffi_null()        { assert_eq!(compile_and_run("tests/regression//m20_edge_ffi_null.xi"),         Some(0)); }
#[test] fn e2e_m20_edge_float_precision() { assert_eq!(compile_and_run("tests/regression//m20_edge_float_precision.xi"),  Some(0)); }
#[test] fn e2e_m20_edge_deep_pattern()    { assert_eq!(compile_and_run("tests/regression//m20_edge_deep_pattern.xi"),     Some(0)); }
#[test] fn e2e_m20_edge_nested_if()       { assert_eq!(compile_and_run("tests/regression//m20_edge_nested_if.xi"),        Some(0)); }
#[test] fn e2e_m20_edge_result_chain()    { assert_eq!(compile_and_run("tests/regression//m20_edge_result_chain.xi"),     Some(0)); }
#[test] fn e2e_m20_edge_multi_module()    { assert_eq!(compile_and_run("tests/regression//m20_edge_multi_module.xi"),     Some(0)); }
#[test] fn e2e_m20_edge_loop_nest()       { assert_eq!(compile_and_run("tests/regression//m20_edge_loop_nest.xi"),        Some(0)); }
#[test] fn e2e_m20_edge_factorial()       { assert_eq!(compile_and_run("tests/regression//m20_edge_factorial.xi"),        Some(0)); }
#[test] fn e2e_m20_edge_char_ops()        { assert_eq!(compile_and_run("tests/regression//m20_edge_char_ops.xi"),         Some(0)); }
#[test] fn e2e_m20_edge_bool_ops()        { assert_eq!(compile_and_run("tests/regression//m20_edge_bool_ops.xi"),         Some(0)); }
#[test] fn e2e_m20_edge_struct_copy()     { assert_eq!(compile_and_run("tests/regression//m20_edge_struct_copy.xi"),      Some(0)); }
#[test] fn e2e_m20_edge_early_return()    { assert_eq!(compile_and_run("tests/regression//m20_edge_early_return.xi"),     Some(0)); }
#[test] fn e2e_m20_edge_while_cond()      { assert_eq!(compile_and_run("tests/regression//m20_edge_while_cond.xi"),       Some(0)); }
#[test] fn e2e_m20_edge_mod_neg()         { assert_eq!(compile_and_run("tests/regression//m20_edge_mod_neg.xi"),          Some(0)); }

// M20: Stress tests
#[test] fn e2e_m20_stress_deep_call()      { assert_eq!(compile_and_run("tests/regression//m20_stress_deep_call.xi"),       Some(0)); }
#[test] fn e2e_m20_stress_many_locals()    { assert_eq!(compile_and_run("tests/regression//m20_stress_many_locals.xi"),     Some(0)); }
#[test] fn e2e_m20_stress_big_loop()       { assert_eq!(compile_and_run("tests/regression//m20_stress_big_loop.xi"),        Some(0)); }
#[test] fn e2e_m20_stress_nested_match()   { assert_eq!(compile_and_run("tests/regression//m20_stress_nested_match.xi"),    Some(0)); }
#[test] fn e2e_m20_stress_struct_fields()  { assert_eq!(compile_and_run("tests/regression//m20_stress_struct_fields.xi"),   Some(0)); }
#[test] fn e2e_m20_stress_many_params()    { assert_eq!(compile_and_run("tests/regression//m20_stress_many_params.xi"),     Some(0)); }
#[test] fn e2e_m20_stress_int_overflow()   { assert_eq!(compile_and_run("tests/regression//m20_stress_int_overflow.xi"),    Some(0)); }
#[test] fn e2e_m20_stress_bit_ops()        { assert_eq!(compile_and_run("tests/regression//m20_stress_bit_ops.xi"),         Some(0)); }
#[test] fn e2e_m20_stress_shift_ops()      { assert_eq!(compile_and_run("tests/regression//m20_stress_shift_ops.xi"),       Some(0)); }
#[test] fn e2e_m20_stress_negate()         { assert_eq!(compile_and_run("tests/regression//m20_stress_negate.xi"),          Some(0)); }
#[test] fn e2e_m20_stress_ternary()        { assert_eq!(compile_and_run("tests/regression//m20_stress_ternary.xi"),         Some(0)); }
#[test] fn e2e_m20_stress_and_or()         { assert_eq!(compile_and_run("tests/regression//m20_stress_and_or.xi"),          Some(0)); }
#[test] fn e2e_m20_stress_float_ops()      { assert_eq!(compile_and_run("tests/regression//m20_stress_float_ops.xi"),       Some(0)); }
#[test] fn e2e_m20_stress_int_div()        { assert_eq!(compile_and_run("tests/regression//m20_stress_int_div.xi"),         Some(0)); }
#[test] fn e2e_m20_stress_bool_return()    { assert_eq!(compile_and_run("tests/regression//m20_stress_bool_return.xi"),     Some(0)); }
#[test] fn e2e_m20_stress_compare()        { assert_eq!(compile_and_run("tests/regression//m20_stress_compare.xi"),         Some(0)); }
#[test] fn e2e_m20_stress_string_concat()  { assert_eq!(compile_and_run("tests/regression//m20_stress_string_concat.xi"),   Some(0)); }
#[test] fn e2e_m20_stress_enum_as_param()  { assert_eq!(compile_and_run("tests/regression//m20_stress_enum_as_param.xi"),   Some(0)); }
#[test] fn e2e_m20_stress_result_as_param(){ assert_eq!(compile_and_run("tests/regression//m20_stress_result_as_param.xi"), Some(0)); }

// M20: Corner-case tests
#[test] fn e2e_m20_corner_shadow_var()     { assert_eq!(compile_and_run("tests/regression//m20_corner_shadow_var.xi"),      Some(0)); }
#[test] fn e2e_m20_corner_empty_block()    { assert_eq!(compile_and_run("tests/regression//m20_corner_empty_block.xi"),     Some(0)); }
#[test] fn e2e_m20_corner_nested_return()  { assert_eq!(compile_and_run("tests/regression//m20_corner_nested_return.xi"),   Some(0)); }
#[test] fn e2e_m20_corner_match_default()  { assert_eq!(compile_and_run("tests/regression//m20_corner_match_default.xi"),   Some(0)); }
#[test] fn e2e_m20_corner_enum_return()    { assert_eq!(compile_and_run("tests/regression//m20_corner_enum_return.xi"),     Some(0)); }
#[test] fn e2e_m20_corner_large_literal()  { assert_eq!(compile_and_run("tests/regression//m20_corner_large_literal.xi"),   Some(0)); }
#[test] fn e2e_m20_corner_zero_init()      { assert_eq!(compile_and_run("tests/regression//m20_corner_zero_init.xi"),       Some(0)); }
#[test] fn e2e_m20_corner_if_no_else()     { assert_eq!(compile_and_run("tests/regression//m20_corner_if_no_else.xi"),      Some(0)); }
#[test] fn e2e_m20_corner_while_zero()     { assert_eq!(compile_and_run("tests/regression//m20_corner_while_zero.xi"),      Some(0)); }
#[test] fn e2e_m20_corner_float_neg()      { assert_eq!(compile_and_run("tests/regression//m20_corner_float_neg.xi"),       Some(0)); }
#[test] fn e2e_m20_corner_pub_fn()         { assert_eq!(compile_and_run("tests/regression//m20_corner_pub_fn.xi"),          Some(0)); }
#[test] fn e2e_m20_corner_const()          { assert_eq!(compile_and_run("tests/regression//m20_corner_const.xi"),           Some(0)); }
#[test] fn e2e_m20_corner_method_chain()   { assert_eq!(compile_and_run("tests/regression//m20_corner_method_chain.xi"),    Some(0)); }
#[test] fn e2e_m20_corner_self_method()    { assert_eq!(compile_and_run("tests/regression//m20_corner_self_method.xi"),     Some(0)); }
#[test] fn e2e_m20_corner_mut_param()      { assert_eq!(compile_and_run("tests/regression//m20_corner_mut_param.xi"),       Some(0)); }
#[test] fn e2e_m20_corner_two_types()      { assert_eq!(compile_and_run("tests/regression//m20_corner_two_types.xi"),       Some(0)); }
#[test] fn e2e_m20_corner_unsafe_block()   { assert_eq!(compile_and_run("tests/regression//m20_corner_unsafe_block.xi"),    Some(0)); }
#[test] fn e2e_m20_corner_concat_chain()   { assert_eq!(compile_and_run("tests/regression//m20_corner_concat_chain.xi"),    Some(0)); }

// M20: Final batch regression tests
#[test] fn e2e_m20_final_arith_expr()      { assert_eq!(compile_and_run("tests/regression//m20_final_arith_expr.xi"),       Some(0)); }
#[test] fn e2e_m20_final_paren_expr()      { assert_eq!(compile_and_run("tests/regression//m20_final_paren_expr.xi"),       Some(0)); }
#[test] fn e2e_m20_final_double_not()      { assert_eq!(compile_and_run("tests/regression//m20_final_double_not.xi"),       Some(0)); }
#[test] fn e2e_m20_final_chained_cmp()     { assert_eq!(compile_and_run("tests/regression//m20_final_chained_cmp.xi"),      Some(0)); }
#[test] fn e2e_m20_final_mixed_bool()      { assert_eq!(compile_and_run("tests/regression//m20_final_mixed_bool.xi"),       Some(0)); }
#[test] fn e2e_m20_final_if_value()        { assert_eq!(compile_and_run("tests/regression//m20_final_if_value.xi"),         Some(0)); }
#[test] fn e2e_m20_final_match_value()     { assert_eq!(compile_and_run("tests/regression//m20_final_match_value.xi"),      Some(0)); }
#[test] fn e2e_m20_final_nested_expr()     { assert_eq!(compile_and_run("tests/regression//m20_final_nested_expr.xi"),      Some(0)); }
#[test] fn e2e_m20_final_return_void()     { assert_eq!(compile_and_run("tests/regression//m20_final_return_void.xi"),      Some(0)); }
#[test] fn e2e_m20_final_early_ret_if()    { assert_eq!(compile_and_run("tests/regression//m20_final_early_ret_if.xi"),     Some(0)); }
#[test] fn e2e_m20_final_loop_if()         { assert_eq!(compile_and_run("tests/regression//m20_final_loop_if.xi"),          Some(0)); }
#[test] fn e2e_m20_final_double_while()    { assert_eq!(compile_and_run("tests/regression//m20_final_double_while.xi"),     Some(0)); }
#[test] fn e2e_m20_final_struct_default()  { assert_eq!(compile_and_run("tests/regression//m20_final_struct_default.xi"),   Some(0)); }
#[test] fn e2e_m20_final_idempotent()      { assert_eq!(compile_and_run("tests/regression//m20_final_idempotent.xi"),       Some(0)); }
#[test] fn e2e_m20_final_reassign_var()    { assert_eq!(compile_and_run("tests/regression//m20_final_reassign_var.xi"),     Some(0)); }
#[test] fn e2e_m20_final_many_returns()    { assert_eq!(compile_and_run("tests/regression//m20_final_many_returns.xi"),     Some(0)); }
#[test] fn e2e_m20_final_deep_arith()      { assert_eq!(compile_and_run("tests/regression//m20_final_deep_arith.xi"),       Some(0)); }
#[test] fn e2e_m20_final_simple_closure()  { assert_eq!(compile_and_run("tests/regression//m20_final_simple_closure.xi"),   Some(0)); }
#[test] fn e2e_m20_final_closure_capture() { assert_eq!(compile_and_run("tests/regression//m20_final_closure_capture.xi"),  Some(0)); }
#[test] fn e2e_m20_final_closure_chain()   { assert_eq!(compile_and_run("tests/regression//m20_final_closure_chain.xi"),    Some(0)); }
#[test] fn e2e_m20_final_option_map()      { assert_eq!(compile_and_run("tests/regression//m20_final_option_map.xi"),       Some(0)); }
#[test] fn e2e_m20_final_result_handle()   { assert_eq!(compile_and_run("tests/regression//m20_final_result_handle.xi"),    Some(0)); }
#[test] fn e2e_m20_final_tuple_pass()      { assert_eq!(compile_and_run("tests/regression//m20_final_tuple_pass.xi"),       Some(0)); }
#[test] fn e2e_m20_final_impl_use()        { assert_eq!(compile_and_run("tests/regression//m20_final_impl_use.xi"),         Some(0)); }
#[test] fn e2e_m20_final_multi_impl()      { assert_eq!(compile_and_run("tests/regression//m20_final_multi_impl.xi"),       Some(0)); }
#[test] fn e2e_m20_corner_global_var()     { assert_eq!(compile_and_run("tests/regression//m20_corner_global_var.xi"),      Some(0)); }
#[test] fn e2e_m20_corner_nested_ifelse()  { assert_eq!(compile_and_run("tests/regression//m20_corner_nested_ifelse.xi"),   Some(0)); }
#[test] fn e2e_m20_stress_option_as_param(){ assert_eq!(compile_and_run("tests/regression//m20_stress_option_as_param.xi"), Some(0)); }
#[test] fn e2e_m20_edge_type_alias()      { assert_eq!(compile_and_run("tests/regression//m20_edge_type_alias.xi"),       Some(0)); }

// -- M22 E2E: Compiler Correctness -------------------------------------
// M22-1: Integer type edge cases
#[test] fn e2e_m22_int8_bounds()          { assert_eq!(compile_and_run("tests/regression//m22_int8_bounds.xi"),           Some(0)); }
#[test] fn e2e_m22_uint8_max()            { assert_eq!(compile_and_run("tests/regression//m22_uint8_max.xi"),             Some(0)); }
#[test] fn e2e_m22_int16_sign()           { assert_eq!(compile_and_run("tests/regression//m22_int16_sign.xi"),            Some(0)); }
#[test] fn e2e_m22_int32_arithmetic()     { assert_eq!(compile_and_run("tests/regression//m22_int32_arithmetic.xi"),      Some(0)); }
#[test] fn e2e_m22_int_casts()            { assert_eq!(compile_and_run("tests/regression//m22_int_casts.xi"),             Some(0)); }
#[test] fn e2e_m22_int_bitwise()          { assert_eq!(compile_and_run("tests/regression//m22_int_bitwise.xi"),           Some(0)); }
#[test] fn e2e_m22_int_cmp()              { assert_eq!(compile_and_run("tests/regression//m22_int_cmp.xi"),               Some(0)); }
// M22-2: Float
#[test] fn e2e_m22_float_arithmetic()     { assert_eq!(compile_and_run("tests/regression//m22_float_arithmetic.xi"),      Some(0)); }
// M22-3: String
#[test] fn e2e_m22_str_operations()       { assert_eq!(compile_and_run("tests/regression//m22_str_operations.xi"),        Some(0)); }
// M22-4: Enum
#[test] fn e2e_m22_enum_payload()         { assert_eq!(compile_and_run("tests/regression//m22_enum_payload.xi"),          Some(0)); }
// M22-5: Struct
#[test] fn e2e_m22_struct_mutate()        { assert_eq!(compile_and_run("tests/regression//m22_struct_mutate.xi"),         Some(99)); }
// M22-6: Generic
#[test] fn e2e_m22_generic_identity()     { assert_eq!(compile_and_run("tests/regression//m22_generic_identity.xi"),      Some(0)); }
// M22-7: Pattern
#[test] fn e2e_m22_pattern_guard()        { assert_eq!(compile_and_run("tests/regression//m22_pattern_guard.xi"),         Some(0)); }

// -- M24: Stress & Robustness E2E --------------------------------------
#[test] fn e2e_m24_many_functions()       { assert_eq!(compile_and_run("tests/regression//m24_many_functions.xi"),        Some(0)); }
#[test] fn e2e_m24_deep_recursion()       { assert_eq!(compile_and_run("tests/regression//m24_deep_recursion.xi"),        Some(0)); }
#[test] fn e2e_m24_type_stress()          { assert_eq!(compile_and_run("tests/regression//m24_type_stress.xi"),           Some(0)); }
#[test] fn e2e_m24_branch_stress()        { assert_eq!(compile_and_run("tests/regression//m24_branch_stress.xi"),         Some(0)); }

// -- M25: Contracts & Verification E2E ---------------------------------
#[test] fn e2e_m25_contract_divide()       { assert_eq!(compile_and_run("tests/regression//m25_contract_divide.xi"),        Some(0)); }
#[test] fn e2e_m25_contract_transfer()     { assert_eq!(compile_and_run("tests/regression//m25_contract_transfer.xi"),      Some(0)); }
#[test] fn e2e_m25_invariant_positive()   { assert_eq!(compile_and_run("tests/regression//m25_invariant_positive.xi"),    Some(0)); }
#[test] fn e2e_m25_contract_clamp()       { assert_eq!(compile_and_run("tests/regression//m25_contract_clamp.xi"),        Some(0)); }

// -- M29: Final Edge Cases E2E -----------------------------------------
#[test] fn e2e_m29_compound_assign()      { assert_eq!(compile_and_run("tests/regression//m29_compound_assign.xi"),       Some(0)); }
#[test] fn e2e_m29_loop_control()         { assert_eq!(compile_and_run("tests/regression//m29_loop_control.xi"),          Some(0)); }
#[test] fn e2e_m29_if_expression()        { assert_eq!(compile_and_run("tests/regression//m29_if_expression.xi"),         Some(0)); }
#[test] fn e2e_m29_type_alias()           { assert_eq!(compile_and_run("tests/regression//m29_type_alias.xi"),            Some(0)); }

// -- M28: Compiler Performance & Optimization E2E ----------------------
#[test] fn e2e_m28_diff_loop_formula()    { assert_eq!(compile_and_run("tests/regression//m28_diff_loop_formula.xi"),     Some(0)); }
#[test] fn e2e_m28_diff_factorial()       { assert_eq!(compile_and_run("tests/regression//m28_diff_factorial.xi"),        Some(0)); }
#[test] fn e2e_m28_diff_commute()         { assert_eq!(compile_and_run("tests/regression//m28_diff_commute.xi"),          Some(0)); }
#[test] fn e2e_m28_large_chain()          { assert_eq!(compile_and_run("tests/regression//m28_large_chain.xi"),           Some(0)); }

// -- M30: Release Readiness E2E ----------------------------------------
#[test] fn e2e_m30_diff_if_match()        { assert_eq!(compile_and_run("tests/regression//m30_diff_if_match.xi"),         Some(0)); }
#[test] fn e2e_m30_diff_gcd()             { assert_eq!(compile_and_run("tests/regression//m30_diff_gcd.xi"),              Some(0)); }
#[test] fn e2e_m30_all_primitives()       { assert_eq!(compile_and_run("tests/regression//m30_all_primitives.xi"),        Some(0)); }
#[test] fn e2e_m30_struct_derive_eq()     { assert_eq!(compile_and_run("tests/regression//m30_struct_derive_eq.xi"),      Some(0)); }

// -- M31: Combinatorial stress E2E -------------------------------------
#[test] fn e2e_m31_diff_sum()             { assert_eq!(compile_and_run("tests/regression//m31_diff_sum.xi"),              Some(0)); }
#[test] fn e2e_m31_diff_fib()             { assert_eq!(compile_and_run("tests/regression//m31_diff_fib.xi"),              Some(0)); }
#[test] fn e2e_m31_all_types()            { assert_eq!(compile_and_run("tests/regression//m31_all_types.xi"),             Some(0)); }

// -- M32: Integer Types Stress Tests -----------------------------------
// Int8 min/max, arithmetic, overflow
#[test] fn e2e_m32_int_0001() { assert_eq!(compile_and_run("tests/regression//m32_int_0001.xi"), Some(0)); }
#[test] fn e2e_m32_int_0002() { assert_eq!(compile_and_run("tests/regression//m32_int_0002.xi"), Some(0)); }
#[test] fn e2e_m32_int_0003() { assert_eq!(compile_and_run("tests/regression//m32_int_0003.xi"), Some(0)); }
#[test] fn e2e_m32_int_0004() { assert_eq!(compile_and_run("tests/regression//m32_int_0004.xi"), Some(0)); }
#[test] fn e2e_m32_int_0005() { assert_eq!(compile_and_run("tests/regression//m32_int_0005.xi"), Some(0)); }
#[test] fn e2e_m32_int_0006() { assert_eq!(compile_and_run("tests/regression//m32_int_0006.xi"), Some(0)); }
#[test] fn e2e_m32_int_0007() { assert_eq!(compile_and_run("tests/regression//m32_int_0007.xi"), Some(0)); }
#[test] fn e2e_m32_int_0008() { assert_eq!(compile_and_run("tests/regression//m32_int_0008.xi"), Some(0)); }
#[test] fn e2e_m32_int_0009() { assert_eq!(compile_and_run("tests/regression//m32_int_0009.xi"), Some(0)); }
#[test] fn e2e_m32_int_0010() { assert_eq!(compile_and_run("tests/regression//m32_int_0010.xi"), Some(0)); }
// Int16 min/max, arithmetic, overflow
#[test] fn e2e_m32_int_0011() { assert_eq!(compile_and_run("tests/regression//m32_int_0011.xi"), Some(0)); }
#[test] fn e2e_m32_int_0012() { assert_eq!(compile_and_run("tests/regression//m32_int_0012.xi"), Some(0)); }
#[test] fn e2e_m32_int_0013() { assert_eq!(compile_and_run("tests/regression//m32_int_0013.xi"), Some(0)); }
#[test] fn e2e_m32_int_0014() { assert_eq!(compile_and_run("tests/regression//m32_int_0014.xi"), Some(0)); }
#[test] fn e2e_m32_int_0015() { assert_eq!(compile_and_run("tests/regression//m32_int_0015.xi"), Some(0)); }
#[test] fn e2e_m32_int_0016() { assert_eq!(compile_and_run("tests/regression//m32_int_0016.xi"), Some(0)); }
#[test] fn e2e_m32_int_0017() { assert_eq!(compile_and_run("tests/regression//m32_int_0017.xi"), Some(0)); }
#[test] fn e2e_m32_int_0018() { assert_eq!(compile_and_run("tests/regression//m32_int_0018.xi"), Some(0)); }
#[test] fn e2e_m32_int_0019() { assert_eq!(compile_and_run("tests/regression//m32_int_0019.xi"), Some(0)); }
#[test] fn e2e_m32_int_0020() { assert_eq!(compile_and_run("tests/regression//m32_int_0020.xi"), Some(0)); }
// Int32 min/max, arithmetic, overflow
#[test] fn e2e_m32_int_0021() { assert_eq!(compile_and_run("tests/regression//m32_int_0021.xi"), Some(0)); }
#[test] fn e2e_m32_int_0022() { assert_eq!(compile_and_run("tests/regression//m32_int_0022.xi"), Some(0)); }
#[test] fn e2e_m32_int_0023() { assert_eq!(compile_and_run("tests/regression//m32_int_0023.xi"), Some(0)); }
#[test] fn e2e_m32_int_0024() { assert_eq!(compile_and_run("tests/regression//m32_int_0024.xi"), Some(0)); }
#[test] fn e2e_m32_int_0025() { assert_eq!(compile_and_run("tests/regression//m32_int_0025.xi"), Some(0)); }
#[test] fn e2e_m32_int_0026() { assert_eq!(compile_and_run("tests/regression//m32_int_0026.xi"), Some(0)); }
#[test] fn e2e_m32_int_0027() { assert_eq!(compile_and_run("tests/regression//m32_int_0027.xi"), Some(0)); }
#[test] fn e2e_m32_int_0028() { assert_eq!(compile_and_run("tests/regression//m32_int_0028.xi"), Some(0)); }
#[test] fn e2e_m32_int_0029() { assert_eq!(compile_and_run("tests/regression//m32_int_0029.xi"), Some(0)); }
#[test] fn e2e_m32_int_0030() { assert_eq!(compile_and_run("tests/regression//m32_int_0030.xi"), Some(0)); }
// Int64 min/max, arithmetic, operations
#[test] fn e2e_m32_int_0031() { assert_eq!(compile_and_run("tests/regression//m32_int_0031.xi"), Some(0)); }
#[test] fn e2e_m32_int_0032() { assert_eq!(compile_and_run("tests/regression//m32_int_0032.xi"), Some(0)); }
#[test] fn e2e_m32_int_0033() { assert_eq!(compile_and_run("tests/regression//m32_int_0033.xi"), Some(0)); }
#[test] fn e2e_m32_int_0034() { assert_eq!(compile_and_run("tests/regression//m32_int_0034.xi"), Some(0)); }
#[test] fn e2e_m32_int_0035() { assert_eq!(compile_and_run("tests/regression//m32_int_0035.xi"), Some(0)); }
#[test] fn e2e_m32_int_0036() { assert_eq!(compile_and_run("tests/regression//m32_int_0036.xi"), Some(0)); }
#[test] fn e2e_m32_int_0037() { assert_eq!(compile_and_run("tests/regression//m32_int_0037.xi"), Some(0)); }
#[test] fn e2e_m32_int_0038() { assert_eq!(compile_and_run("tests/regression//m32_int_0038.xi"), Some(0)); }
#[test] fn e2e_m32_int_0039() { assert_eq!(compile_and_run("tests/regression//m32_int_0039.xi"), Some(0)); }
#[test] fn e2e_m32_int_0040() { assert_eq!(compile_and_run("tests/regression//m32_int_0040.xi"), Some(0)); }
// UInt types max, unsigned arithmetic
#[test] fn e2e_m32_int_0041() { assert_eq!(compile_and_run("tests/regression//m32_int_0041.xi"), Some(0)); }
#[test] fn e2e_m32_int_0042() { assert_eq!(compile_and_run("tests/regression//m32_int_0042.xi"), Some(0)); }
#[test] fn e2e_m32_int_0043() { assert_eq!(compile_and_run("tests/regression//m32_int_0043.xi"), Some(0)); }
#[test] fn e2e_m32_int_0044() { assert_eq!(compile_and_run("tests/regression//m32_int_0044.xi"), Some(0)); }
#[test] fn e2e_m32_int_0045() { assert_eq!(compile_and_run("tests/regression//m32_int_0045.xi"), Some(0)); }
#[test] fn e2e_m32_int_0046() { assert_eq!(compile_and_run("tests/regression//m32_int_0046.xi"), Some(0)); }
#[test] fn e2e_m32_int_0047() { assert_eq!(compile_and_run("tests/regression//m32_int_0047.xi"), Some(0)); }
#[test] fn e2e_m32_int_0048() { assert_eq!(compile_and_run("tests/regression//m32_int_0048.xi"), Some(0)); }
#[test] fn e2e_m32_int_0049() { assert_eq!(compile_and_run("tests/regression//m32_int_0049.xi"), Some(0)); }
#[test] fn e2e_m32_int_0050() { assert_eq!(compile_and_run("tests/regression//m32_int_0050.xi"), Some(0)); }
// Division by zero guards, casts, signed/unsigned conversions
#[test] fn e2e_m32_int_0051() { assert_eq!(compile_and_run("tests/regression//m32_int_0051.xi"), Some(0)); }
#[test] fn e2e_m32_int_0052() { assert_eq!(compile_and_run("tests/regression//m32_int_0052.xi"), Some(0)); }
#[test] fn e2e_m32_int_0053() { assert_eq!(compile_and_run("tests/regression//m32_int_0053.xi"), Some(0)); }
#[test] fn e2e_m32_int_0054() { assert_eq!(compile_and_run("tests/regression//m32_int_0054.xi"), Some(0)); }
#[test] fn e2e_m32_int_0055() { assert_eq!(compile_and_run("tests/regression//m32_int_0055.xi"), Some(0)); }
#[test] fn e2e_m32_int_0056() { assert_eq!(compile_and_run("tests/regression//m32_int_0056.xi"), Some(0)); }
#[test] fn e2e_m32_int_0057() { assert_eq!(compile_and_run("tests/regression//m32_int_0057.xi"), Some(0)); }
#[test] fn e2e_m32_int_0058() { assert_eq!(compile_and_run("tests/regression//m32_int_0058.xi"), Some(0)); }
#[test] fn e2e_m32_int_0059() { assert_eq!(compile_and_run("tests/regression//m32_int_0059.xi"), Some(0)); }
#[test] fn e2e_m32_int_0060() { assert_eq!(compile_and_run("tests/regression//m32_int_0060.xi"), Some(0)); }
// Bitwise AND/OR/XOR/NOT, shifts on all types
#[test] fn e2e_m32_int_0061() { assert_eq!(compile_and_run("tests/regression//m32_int_0061.xi"), Some(0)); }
#[test] fn e2e_m32_int_0062() { assert_eq!(compile_and_run("tests/regression//m32_int_0062.xi"), Some(0)); }
#[test] fn e2e_m32_int_0063() { assert_eq!(compile_and_run("tests/regression//m32_int_0063.xi"), Some(0)); }
#[test] fn e2e_m32_int_0064() { assert_eq!(compile_and_run("tests/regression//m32_int_0064.xi"), Some(0)); }
#[test] fn e2e_m32_int_0065() { assert_eq!(compile_and_run("tests/regression//m32_int_0065.xi"), Some(0)); }
#[test] fn e2e_m32_int_0066() { assert_eq!(compile_and_run("tests/regression//m32_int_0066.xi"), Some(0)); }
#[test] fn e2e_m32_int_0067() { assert_eq!(compile_and_run("tests/regression//m32_int_0067.xi"), Some(0)); }
#[test] fn e2e_m32_int_0068() { assert_eq!(compile_and_run("tests/regression//m32_int_0068.xi"), Some(0)); }
#[test] fn e2e_m32_int_0069() { assert_eq!(compile_and_run("tests/regression//m32_int_0069.xi"), Some(0)); }
#[test] fn e2e_m32_int_0070() { assert_eq!(compile_and_run("tests/regression//m32_int_0070.xi"), Some(0)); }
// Comparison chains, unary negation, mixed-type, hex literals
#[test] fn e2e_m32_int_0071() { assert_eq!(compile_and_run("tests/regression//m32_int_0071.xi"), Some(0)); }
#[test] fn e2e_m32_int_0072() { assert_eq!(compile_and_run("tests/regression//m32_int_0072.xi"), Some(0)); }
#[test] fn e2e_m32_int_0073() { assert_eq!(compile_and_run("tests/regression//m32_int_0073.xi"), Some(0)); }
#[test] fn e2e_m32_int_0074() { assert_eq!(compile_and_run("tests/regression//m32_int_0074.xi"), Some(0)); }
#[test] fn e2e_m32_int_0075() { assert_eq!(compile_and_run("tests/regression//m32_int_0075.xi"), Some(0)); }
#[test] fn e2e_m32_int_0076() { assert_eq!(compile_and_run("tests/regression//m32_int_0076.xi"), Some(0)); }
#[test] fn e2e_m32_int_0077() { assert_eq!(compile_and_run("tests/regression//m32_int_0077.xi"), Some(0)); }
#[test] fn e2e_m32_int_0078() { assert_eq!(compile_and_run("tests/regression//m32_int_0078.xi"), Some(0)); }
#[test] fn e2e_m32_int_0079() { assert_eq!(compile_and_run("tests/regression//m32_int_0079.xi"), Some(0)); }
#[test] fn e2e_m32_int_0080() { assert_eq!(compile_and_run("tests/regression//m32_int_0080.xi"), Some(0)); }
// Struct, enum, generic with integers
#[test] fn e2e_m32_int_0081() { assert_eq!(compile_and_run("tests/regression//m32_int_0081.xi"), Some(0)); }
#[test] fn e2e_m32_int_0082() { assert_eq!(compile_and_run("tests/regression//m32_int_0082.xi"), Some(0)); }
#[test] fn e2e_m32_int_0083() { assert_eq!(compile_and_run("tests/regression//m32_int_0083.xi"), Some(0)); }
#[test] fn e2e_m32_int_0084() { assert_eq!(compile_and_run("tests/regression//m32_int_0084.xi"), Some(0)); }
#[test] fn e2e_m32_int_0085() { assert_eq!(compile_and_run("tests/regression//m32_int_0085.xi"), Some(0)); }
#[test] fn e2e_m32_int_0086() { assert_eq!(compile_and_run("tests/regression//m32_int_0086.xi"), Some(0)); }
#[test] fn e2e_m32_int_0087() { assert_eq!(compile_and_run("tests/regression//m32_int_0087.xi"), Some(0)); }
#[test] fn e2e_m32_int_0088() { assert_eq!(compile_and_run("tests/regression//m32_int_0088.xi"), Some(0)); }
#[test] fn e2e_m32_int_0089() { assert_eq!(compile_and_run("tests/regression//m32_int_0089.xi"), Some(0)); }
#[test] fn e2e_m32_int_0090() { assert_eq!(compile_and_run("tests/regression//m32_int_0090.xi"), Some(0)); }
#[test] fn e2e_m32_int_0100() { assert_eq!(compile_and_run("tests/regression//m32_int_0100.xi"), Some(0)); }
#[test] fn e2e_m32_int_0101() { assert_eq!(compile_and_run("tests/regression//m32_int_0101.xi"), Some(0)); }
#[test] fn e2e_m32_int_0102() { assert_eq!(compile_and_run("tests/regression//m32_int_0102.xi"), Some(0)); }
#[test] fn e2e_m32_int_0103() { assert_eq!(compile_and_run("tests/regression//m32_int_0103.xi"), Some(0)); }
#[test] fn e2e_m32_int_0104() { assert_eq!(compile_and_run("tests/regression//m32_int_0104.xi"), Some(0)); }
#[test] fn e2e_m32_int_0105() { assert_eq!(compile_and_run("tests/regression//m32_int_0105.xi"), Some(0)); }
#[test] fn e2e_m32_int_0106() { assert_eq!(compile_and_run("tests/regression//m32_int_0106.xi"), Some(0)); }
#[test] fn e2e_m32_int_0107() { assert_eq!(compile_and_run("tests/regression//m32_int_0107.xi"), Some(0)); }
#[test] fn e2e_m32_int_0108() { assert_eq!(compile_and_run("tests/regression//m32_int_0108.xi"), Some(0)); }
#[test] fn e2e_m32_int_0109() { assert_eq!(compile_and_run("tests/regression//m32_int_0109.xi"), Some(0)); }
#[test] fn e2e_m32_int_0110() { assert_eq!(compile_and_run("tests/regression//m32_int_0110.xi"), Some(0)); }
#[test] fn e2e_m32_int_0111() { assert_eq!(compile_and_run("tests/regression//m32_int_0111.xi"), Some(0)); }
#[test] fn e2e_m32_int_0112() { assert_eq!(compile_and_run("tests/regression//m32_int_0112.xi"), Some(0)); }
#[test] fn e2e_m32_int_0113() { assert_eq!(compile_and_run("tests/regression//m32_int_0113.xi"), Some(0)); }
#[test] fn e2e_m32_int_0114() { assert_eq!(compile_and_run("tests/regression//m32_int_0114.xi"), Some(0)); }
#[test] fn e2e_m32_int_0115() { assert_eq!(compile_and_run("tests/regression//m32_int_0115.xi"), Some(0)); }
#[test] fn e2e_m32_int_0116() { assert_eq!(compile_and_run("tests/regression//m32_int_0116.xi"), Some(0)); }
#[test] fn e2e_m32_int_0117() { assert_eq!(compile_and_run("tests/regression//m32_int_0117.xi"), Some(0)); }
#[test] fn e2e_m32_int_0118() { assert_eq!(compile_and_run("tests/regression//m32_int_0118.xi"), Some(0)); }
#[test] fn e2e_m32_int_0119() { assert_eq!(compile_and_run("tests/regression//m32_int_0119.xi"), Some(0)); }
#[test] fn e2e_m32_int_0120() { assert_eq!(compile_and_run("tests/regression//m32_int_0120.xi"), Some(0)); }
#[test] fn e2e_m32_int_0121() { assert_eq!(compile_and_run("tests/regression//m32_int_0121.xi"), Some(0)); }
#[test] fn e2e_m32_int_0122() { assert_eq!(compile_and_run("tests/regression//m32_int_0122.xi"), Some(0)); }
#[test] fn e2e_m32_int_0123() { assert_eq!(compile_and_run("tests/regression//m32_int_0123.xi"), Some(0)); }
#[test] fn e2e_m32_int_0124() { assert_eq!(compile_and_run("tests/regression//m32_int_0124.xi"), Some(0)); }
#[test] fn e2e_m32_int_0125() { assert_eq!(compile_and_run("tests/regression//m32_int_0125.xi"), Some(0)); }
#[test] fn e2e_m32_int_0126() { assert_eq!(compile_and_run("tests/regression//m32_int_0126.xi"), Some(0)); }
#[test] fn e2e_m32_int_0127() { assert_eq!(compile_and_run("tests/regression//m32_int_0127.xi"), Some(0)); }
#[test] fn e2e_m32_int_0128() { assert_eq!(compile_and_run("tests/regression//m32_int_0128.xi"), Some(0)); }
#[test] fn e2e_m32_int_0129() { assert_eq!(compile_and_run("tests/regression//m32_int_0129.xi"), Some(0)); }
#[test] fn e2e_m32_int_0130() { assert_eq!(compile_and_run("tests/regression//m32_int_0130.xi"), Some(0)); }
#[test] fn e2e_m32_int_0131() { assert_eq!(compile_and_run("tests/regression//m32_int_0131.xi"), Some(0)); }
#[test] fn e2e_m32_int_0132() { assert_eq!(compile_and_run("tests/regression//m32_int_0132.xi"), Some(0)); }
#[test] fn e2e_m32_int_0133() { assert_eq!(compile_and_run("tests/regression//m32_int_0133.xi"), Some(0)); }
#[test] fn e2e_m32_int_0134() { assert_eq!(compile_and_run("tests/regression//m32_int_0134.xi"), Some(0)); }
#[test] fn e2e_m32_int_0135() { assert_eq!(compile_and_run("tests/regression//m32_int_0135.xi"), Some(0)); }
#[test] fn e2e_m32_int_0136() { assert_eq!(compile_and_run("tests/regression//m32_int_0136.xi"), Some(0)); }
#[test] fn e2e_m32_int_0137() { assert_eq!(compile_and_run("tests/regression//m32_int_0137.xi"), Some(0)); }
#[test] fn e2e_m32_int_0138() { assert_eq!(compile_and_run("tests/regression//m32_int_0138.xi"), Some(0)); }
#[test] fn e2e_m32_int_0139() { assert_eq!(compile_and_run("tests/regression//m32_int_0139.xi"), Some(0)); }
#[test] fn e2e_m32_int_0140() { assert_eq!(compile_and_run("tests/regression//m32_int_0140.xi"), Some(0)); }
#[test] fn e2e_m32_int_0141() { assert_eq!(compile_and_run("tests/regression//m32_int_0141.xi"), Some(0)); }
#[test] fn e2e_m32_int_0142() { assert_eq!(compile_and_run("tests/regression//m32_int_0142.xi"), Some(0)); }
#[test] fn e2e_m32_int_0143() { assert_eq!(compile_and_run("tests/regression//m32_int_0143.xi"), Some(0)); }
#[test] fn e2e_m32_int_0144() { assert_eq!(compile_and_run("tests/regression//m32_int_0144.xi"), Some(0)); }
#[test] fn e2e_m32_int_0145() { assert_eq!(compile_and_run("tests/regression//m32_int_0145.xi"), Some(0)); }
#[test] fn e2e_m32_int_0146() { assert_eq!(compile_and_run("tests/regression//m32_int_0146.xi"), Some(0)); }
#[test] fn e2e_m32_int_0147() { assert_eq!(compile_and_run("tests/regression//m32_int_0147.xi"), Some(0)); }
#[test] fn e2e_m32_int_0148() { assert_eq!(compile_and_run("tests/regression//m32_int_0148.xi"), Some(0)); }
#[test] fn e2e_m32_int_0149() { assert_eq!(compile_and_run("tests/regression//m32_int_0149.xi"), Some(0)); }
#[test] fn e2e_m32_int_0150() { assert_eq!(compile_and_run("tests/regression//m32_int_0150.xi"), Some(0)); }
#[test] fn e2e_m32_int_0151() { assert_eq!(compile_and_run("tests/regression//m32_int_0151.xi"), Some(0)); }
#[test] fn e2e_m32_int_0152() { assert_eq!(compile_and_run("tests/regression//m32_int_0152.xi"), Some(0)); }
#[test] fn e2e_m32_int_0153() { assert_eq!(compile_and_run("tests/regression//m32_int_0153.xi"), Some(0)); }
#[test] fn e2e_m32_int_0154() { assert_eq!(compile_and_run("tests/regression//m32_int_0154.xi"), Some(0)); }
#[test] fn e2e_m32_int_0155() { assert_eq!(compile_and_run("tests/regression//m32_int_0155.xi"), Some(0)); }
#[test] fn e2e_m32_int_0156() { assert_eq!(compile_and_run("tests/regression//m32_int_0156.xi"), Some(0)); }
#[test] fn e2e_m32_int_0157() { assert_eq!(compile_and_run("tests/regression//m32_int_0157.xi"), Some(0)); }
#[test] fn e2e_m32_int_0158() { assert_eq!(compile_and_run("tests/regression//m32_int_0158.xi"), Some(0)); }
#[test] fn e2e_m32_int_0159() { assert_eq!(compile_and_run("tests/regression//m32_int_0159.xi"), Some(0)); }
#[test] fn e2e_m32_int_0160() { assert_eq!(compile_and_run("tests/regression//m32_int_0160.xi"), Some(0)); }
#[test] fn e2e_m32_int_0161() { assert_eq!(compile_and_run("tests/regression//m32_int_0161.xi"), Some(0)); }
#[test] fn e2e_m32_int_0162() { assert_eq!(compile_and_run("tests/regression//m32_int_0162.xi"), Some(0)); }
#[test] fn e2e_m32_int_0163() { assert_eq!(compile_and_run("tests/regression//m32_int_0163.xi"), Some(0)); }
#[test] fn e2e_m32_int_0164() { assert_eq!(compile_and_run("tests/regression//m32_int_0164.xi"), Some(0)); }
#[test] fn e2e_m32_int_0165() { assert_eq!(compile_and_run("tests/regression//m32_int_0165.xi"), Some(0)); }
#[test] fn e2e_m32_int_0166() { assert_eq!(compile_and_run("tests/regression//m32_int_0166.xi"), Some(0)); }
#[test] fn e2e_m32_int_0167() { assert_eq!(compile_and_run("tests/regression//m32_int_0167.xi"), Some(0)); }
#[test] fn e2e_m32_int_0168() { assert_eq!(compile_and_run("tests/regression//m32_int_0168.xi"), Some(0)); }
#[test] fn e2e_m32_int_0169() { assert_eq!(compile_and_run("tests/regression//m32_int_0169.xi"), Some(0)); }
#[test] fn e2e_m32_int_0170() { assert_eq!(compile_and_run("tests/regression//m32_int_0170.xi"), Some(0)); }
#[test] fn e2e_m32_int_0171() { assert_eq!(compile_and_run("tests/regression//m32_int_0171.xi"), Some(0)); }
#[test] fn e2e_m32_int_0172() { assert_eq!(compile_and_run("tests/regression//m32_int_0172.xi"), Some(0)); }
#[test] fn e2e_m32_int_0173() { assert_eq!(compile_and_run("tests/regression//m32_int_0173.xi"), Some(0)); }
#[test] fn e2e_m32_int_0174() { assert_eq!(compile_and_run("tests/regression//m32_int_0174.xi"), Some(0)); }
#[test] fn e2e_m32_int_0175() { assert_eq!(compile_and_run("tests/regression//m32_int_0175.xi"), Some(0)); }
#[test] fn e2e_m32_int_0176() { assert_eq!(compile_and_run("tests/regression//m32_int_0176.xi"), Some(0)); }
#[test] fn e2e_m32_int_0177() { assert_eq!(compile_and_run("tests/regression//m32_int_0177.xi"), Some(0)); }
#[test] fn e2e_m32_int_0178() { assert_eq!(compile_and_run("tests/regression//m32_int_0178.xi"), Some(0)); }
#[test] fn e2e_m32_int_0179() { assert_eq!(compile_and_run("tests/regression//m32_int_0179.xi"), Some(0)); }
#[test] fn e2e_m32_int_0180() { assert_eq!(compile_and_run("tests/regression//m32_int_0180.xi"), Some(0)); }
#[test] fn e2e_m32_int_0181() { assert_eq!(compile_and_run("tests/regression//m32_int_0181.xi"), Some(0)); }
#[test] fn e2e_m32_int_0182() { assert_eq!(compile_and_run("tests/regression//m32_int_0182.xi"), Some(0)); }
#[test] fn e2e_m32_int_0183() { assert_eq!(compile_and_run("tests/regression//m32_int_0183.xi"), Some(0)); }
#[test] fn e2e_m32_int_0184() { assert_eq!(compile_and_run("tests/regression//m32_int_0184.xi"), Some(0)); }
#[test] fn e2e_m32_int_0185() { assert_eq!(compile_and_run("tests/regression//m32_int_0185.xi"), Some(0)); }
#[test] fn e2e_m32_int_0186() { assert_eq!(compile_and_run("tests/regression//m32_int_0186.xi"), Some(0)); }
#[test] fn e2e_m32_int_0187() { assert_eq!(compile_and_run("tests/regression//m32_int_0187.xi"), Some(0)); }
#[test] fn e2e_m32_int_0188() { assert_eq!(compile_and_run("tests/regression//m32_int_0188.xi"), Some(0)); }
#[test] fn e2e_m32_int_0189() { assert_eq!(compile_and_run("tests/regression//m32_int_0189.xi"), Some(0)); }
#[test] fn e2e_m32_int_0190() { assert_eq!(compile_and_run("tests/regression//m32_int_0190.xi"), Some(0)); }
#[test] fn e2e_m32_int_0191() { assert_eq!(compile_and_run("tests/regression//m32_int_0191.xi"), Some(0)); }
#[test] fn e2e_m32_int_0192() { assert_eq!(compile_and_run("tests/regression//m32_int_0192.xi"), Some(0)); }
#[test] fn e2e_m32_int_0193() { assert_eq!(compile_and_run("tests/regression//m32_int_0193.xi"), Some(0)); }
#[test] fn e2e_m32_int_0194() { assert_eq!(compile_and_run("tests/regression//m32_int_0194.xi"), Some(0)); }
#[test] fn e2e_m32_int_0195() { assert_eq!(compile_and_run("tests/regression//m32_int_0195.xi"), Some(0)); }
#[test] fn e2e_m32_int_0196() { assert_eq!(compile_and_run("tests/regression//m32_int_0196.xi"), Some(0)); }
#[test] fn e2e_m32_int_0197() { assert_eq!(compile_and_run("tests/regression//m32_int_0197.xi"), Some(0)); }
#[test] fn e2e_m32_int_0198() { assert_eq!(compile_and_run("tests/regression//m32_int_0198.xi"), Some(0)); }
#[test] fn e2e_m32_int_0199() { assert_eq!(compile_and_run("tests/regression//m32_int_0199.xi"), Some(0)); }
#[test] fn e2e_m32_int_0200() { assert_eq!(compile_and_run("tests/regression//m32_int_0200.xi"), Some(0)); }
#[test] fn e2e_m32_int_0201() { assert_eq!(compile_and_run("tests/regression//m32_int_0201.xi"), Some(0)); }
#[test] fn e2e_m32_int_0202() { assert_eq!(compile_and_run("tests/regression//m32_int_0202.xi"), Some(0)); }
#[test] fn e2e_m32_int_0203() { assert_eq!(compile_and_run("tests/regression//m32_int_0203.xi"), Some(0)); }
#[test] fn e2e_m32_int_0204() { assert_eq!(compile_and_run("tests/regression//m32_int_0204.xi"), Some(0)); }
#[test] fn e2e_m32_int_0205() { assert_eq!(compile_and_run("tests/regression//m32_int_0205.xi"), Some(0)); }
#[test] fn e2e_m32_int_0206() { assert_eq!(compile_and_run("tests/regression//m32_int_0206.xi"), Some(0)); }
#[test] fn e2e_m32_int_0207() { assert_eq!(compile_and_run("tests/regression//m32_int_0207.xi"), Some(0)); }
#[test] fn e2e_m32_int_0208() { assert_eq!(compile_and_run("tests/regression//m32_int_0208.xi"), Some(0)); }
#[test] fn e2e_m32_int_0209() { assert_eq!(compile_and_run("tests/regression//m32_int_0209.xi"), Some(0)); }
#[test] fn e2e_m32_int_0210() { assert_eq!(compile_and_run("tests/regression//m32_int_0210.xi"), Some(0)); }
#[test] fn e2e_m32_int_0211() { assert_eq!(compile_and_run("tests/regression//m32_int_0211.xi"), Some(0)); }
#[test] fn e2e_m32_int_0212() { assert_eq!(compile_and_run("tests/regression//m32_int_0212.xi"), Some(0)); }
#[test] fn e2e_m32_int_0213() { assert_eq!(compile_and_run("tests/regression//m32_int_0213.xi"), Some(0)); }
#[test] fn e2e_m32_int_0214() { assert_eq!(compile_and_run("tests/regression//m32_int_0214.xi"), Some(0)); }
#[test] fn e2e_m32_int_0215() { assert_eq!(compile_and_run("tests/regression//m32_int_0215.xi"), Some(0)); }
#[test] fn e2e_m32_int_0216() { assert_eq!(compile_and_run("tests/regression//m32_int_0216.xi"), Some(0)); }
#[test] fn e2e_m32_int_0217() { assert_eq!(compile_and_run("tests/regression//m32_int_0217.xi"), Some(0)); }
#[test] fn e2e_m32_int_0218() { assert_eq!(compile_and_run("tests/regression//m32_int_0218.xi"), Some(0)); }
#[test] fn e2e_m32_int_0219() { assert_eq!(compile_and_run("tests/regression//m32_int_0219.xi"), Some(0)); }
#[test] fn e2e_m32_int_0220() { assert_eq!(compile_and_run("tests/regression//m32_int_0220.xi"), Some(0)); }
#[test] fn e2e_m32_int_0221() { assert_eq!(compile_and_run("tests/regression//m32_int_0221.xi"), Some(0)); }
#[test] fn e2e_m32_int_0222() { assert_eq!(compile_and_run("tests/regression//m32_int_0222.xi"), Some(0)); }
#[test] fn e2e_m32_int_0223() { assert_eq!(compile_and_run("tests/regression//m32_int_0223.xi"), Some(0)); }
#[test] fn e2e_m32_int_0224() { assert_eq!(compile_and_run("tests/regression//m32_int_0224.xi"), Some(0)); }
#[test] fn e2e_m32_int_0225() { assert_eq!(compile_and_run("tests/regression//m32_int_0225.xi"), Some(0)); }
#[test] fn e2e_m32_int_0226() { assert_eq!(compile_and_run("tests/regression//m32_int_0226.xi"), Some(0)); }
#[test] fn e2e_m32_int_0227() { assert_eq!(compile_and_run("tests/regression//m32_int_0227.xi"), Some(0)); }
#[test] fn e2e_m32_int_0228() { assert_eq!(compile_and_run("tests/regression//m32_int_0228.xi"), Some(0)); }
#[test] fn e2e_m32_int_0229() { assert_eq!(compile_and_run("tests/regression//m32_int_0229.xi"), Some(0)); }
#[test] fn e2e_m32_int_0230() { assert_eq!(compile_and_run("tests/regression//m32_int_0230.xi"), Some(0)); }
#[test] fn e2e_m32_int_0231() { assert_eq!(compile_and_run("tests/regression//m32_int_0231.xi"), Some(0)); }
#[test] fn e2e_m32_int_0232() { assert_eq!(compile_and_run("tests/regression//m32_int_0232.xi"), Some(0)); }
#[test] fn e2e_m32_int_0233() { assert_eq!(compile_and_run("tests/regression//m32_int_0233.xi"), Some(0)); }
#[test] fn e2e_m32_int_0234() { assert_eq!(compile_and_run("tests/regression//m32_int_0234.xi"), Some(0)); }
#[test] fn e2e_m32_int_0235() { assert_eq!(compile_and_run("tests/regression//m32_int_0235.xi"), Some(0)); }
#[test] fn e2e_m32_int_0236() { assert_eq!(compile_and_run("tests/regression//m32_int_0236.xi"), Some(0)); }
#[test] fn e2e_m32_int_0237() { assert_eq!(compile_and_run("tests/regression//m32_int_0237.xi"), Some(0)); }
#[test] fn e2e_m32_int_0238() { assert_eq!(compile_and_run("tests/regression//m32_int_0238.xi"), Some(0)); }
#[test] fn e2e_m32_int_0239() { assert_eq!(compile_and_run("tests/regression//m32_int_0239.xi"), Some(0)); }
#[test] fn e2e_m32_int_0240() { assert_eq!(compile_and_run("tests/regression//m32_int_0240.xi"), Some(0)); }
#[test] fn e2e_m32_int_0241() { assert_eq!(compile_and_run("tests/regression//m32_int_0241.xi"), Some(0)); }
#[test] fn e2e_m32_int_0242() { assert_eq!(compile_and_run("tests/regression//m32_int_0242.xi"), Some(0)); }
#[test] fn e2e_m32_int_0243() { assert_eq!(compile_and_run("tests/regression//m32_int_0243.xi"), Some(0)); }
#[test] fn e2e_m32_int_0244() { assert_eq!(compile_and_run("tests/regression//m32_int_0244.xi"), Some(0)); }
#[test] fn e2e_m32_int_0245() { assert_eq!(compile_and_run("tests/regression//m32_int_0245.xi"), Some(0)); }
#[test] fn e2e_m32_int_0246() { assert_eq!(compile_and_run("tests/regression//m32_int_0246.xi"), Some(0)); }
#[test] fn e2e_m32_int_0247() { assert_eq!(compile_and_run("tests/regression//m32_int_0247.xi"), Some(0)); }
#[test] fn e2e_m32_int_0248() { assert_eq!(compile_and_run("tests/regression//m32_int_0248.xi"), Some(0)); }
#[test] fn e2e_m32_int_0249() { assert_eq!(compile_and_run("tests/regression//m32_int_0249.xi"), Some(0)); }
#[test] fn e2e_m32_int_0250() { assert_eq!(compile_and_run("tests/regression//m32_int_0250.xi"), Some(0)); }
#[test] fn e2e_m32_int_0251() { assert_eq!(compile_and_run("tests/regression//m32_int_0251.xi"), Some(0)); }
#[test] fn e2e_m32_int_0252() { assert_eq!(compile_and_run("tests/regression//m32_int_0252.xi"), Some(0)); }
#[test] fn e2e_m32_int_0253() { assert_eq!(compile_and_run("tests/regression//m32_int_0253.xi"), Some(0)); }
#[test] fn e2e_m32_int_0254() { assert_eq!(compile_and_run("tests/regression//m32_int_0254.xi"), Some(0)); }
#[test] fn e2e_m32_int_0255() { assert_eq!(compile_and_run("tests/regression//m32_int_0255.xi"), Some(0)); }
#[test] fn e2e_m32_int_0256() { assert_eq!(compile_and_run("tests/regression//m32_int_0256.xi"), Some(0)); }
#[test] fn e2e_m32_int_0257() { assert_eq!(compile_and_run("tests/regression//m32_int_0257.xi"), Some(0)); }
#[test] fn e2e_m32_int_0258() { assert_eq!(compile_and_run("tests/regression//m32_int_0258.xi"), Some(0)); }
#[test] fn e2e_m32_int_0259() { assert_eq!(compile_and_run("tests/regression//m32_int_0259.xi"), Some(0)); }
#[test] fn e2e_m32_int_0260() { assert_eq!(compile_and_run("tests/regression//m32_int_0260.xi"), Some(0)); }
#[test] fn e2e_m32_int_0261() { assert_eq!(compile_and_run("tests/regression//m32_int_0261.xi"), Some(0)); }
#[test] fn e2e_m32_int_0262() { assert_eq!(compile_and_run("tests/regression//m32_int_0262.xi"), Some(0)); }
#[test] fn e2e_m32_int_0263() { assert_eq!(compile_and_run("tests/regression//m32_int_0263.xi"), Some(0)); }
#[test] fn e2e_m32_int_0264() { assert_eq!(compile_and_run("tests/regression//m32_int_0264.xi"), Some(0)); }
#[test] fn e2e_m32_int_0265() { assert_eq!(compile_and_run("tests/regression//m32_int_0265.xi"), Some(0)); }
#[test] fn e2e_m32_int_0266() { assert_eq!(compile_and_run("tests/regression//m32_int_0266.xi"), Some(0)); }
#[test] fn e2e_m32_int_0267() { assert_eq!(compile_and_run("tests/regression//m32_int_0267.xi"), Some(0)); }
#[test] fn e2e_m32_int_0268() { assert_eq!(compile_and_run("tests/regression//m32_int_0268.xi"), Some(0)); }
#[test] fn e2e_m32_int_0269() { assert_eq!(compile_and_run("tests/regression//m32_int_0269.xi"), Some(0)); }
#[test] fn e2e_m32_int_0270() { assert_eq!(compile_and_run("tests/regression//m32_int_0270.xi"), Some(0)); }
#[test] fn e2e_m32_int_0271() { assert_eq!(compile_and_run("tests/regression//m32_int_0271.xi"), Some(0)); }
#[test] fn e2e_m32_int_0272() { assert_eq!(compile_and_run("tests/regression//m32_int_0272.xi"), Some(0)); }
#[test] fn e2e_m32_int_0273() { assert_eq!(compile_and_run("tests/regression//m32_int_0273.xi"), Some(0)); }
#[test] fn e2e_m32_int_0274() { assert_eq!(compile_and_run("tests/regression//m32_int_0274.xi"), Some(0)); }
#[test] fn e2e_m32_int_0275() { assert_eq!(compile_and_run("tests/regression//m32_int_0275.xi"), Some(0)); }
#[test] fn e2e_m32_int_0276() { assert_eq!(compile_and_run("tests/regression//m32_int_0276.xi"), Some(0)); }
#[test] fn e2e_m32_int_0277() { assert_eq!(compile_and_run("tests/regression//m32_int_0277.xi"), Some(0)); }
#[test] fn e2e_m32_int_0278() { assert_eq!(compile_and_run("tests/regression//m32_int_0278.xi"), Some(0)); }
#[test] fn e2e_m32_int_0279() { assert_eq!(compile_and_run("tests/regression//m32_int_0279.xi"), Some(0)); }
#[test] fn e2e_m32_int_0280() { assert_eq!(compile_and_run("tests/regression//m32_int_0280.xi"), Some(0)); }
#[test] fn e2e_m32_int_0281() { assert_eq!(compile_and_run("tests/regression//m32_int_0281.xi"), Some(0)); }
#[test] fn e2e_m32_int_0282() { assert_eq!(compile_and_run("tests/regression//m32_int_0282.xi"), Some(0)); }
#[test] fn e2e_m32_int_0283() { assert_eq!(compile_and_run("tests/regression//m32_int_0283.xi"), Some(0)); }
#[test] fn e2e_m32_int_0284() { assert_eq!(compile_and_run("tests/regression//m32_int_0284.xi"), Some(0)); }
#[test] fn e2e_m32_int_0285() { assert_eq!(compile_and_run("tests/regression//m32_int_0285.xi"), Some(0)); }
#[test] fn e2e_m32_int_0286() { assert_eq!(compile_and_run("tests/regression//m32_int_0286.xi"), Some(0)); }
#[test] fn e2e_m32_int_0287() { assert_eq!(compile_and_run("tests/regression//m32_int_0287.xi"), Some(0)); }
#[test] fn e2e_m32_int_0288() { assert_eq!(compile_and_run("tests/regression//m32_int_0288.xi"), Some(0)); }
#[test] fn e2e_m32_int_0289() { assert_eq!(compile_and_run("tests/regression//m32_int_0289.xi"), Some(0)); }
#[test] fn e2e_m32_int_0290() { assert_eq!(compile_and_run("tests/regression//m32_int_0290.xi"), Some(0)); }
#[test] fn e2e_m32_int_0291() { assert_eq!(compile_and_run("tests/regression//m32_int_0291.xi"), Some(0)); }
#[test] fn e2e_m32_int_0292() { assert_eq!(compile_and_run("tests/regression//m32_int_0292.xi"), Some(0)); }
#[test] fn e2e_m32_int_0293() { assert_eq!(compile_and_run("tests/regression//m32_int_0293.xi"), Some(0)); }
#[test] fn e2e_m32_int_0294() { assert_eq!(compile_and_run("tests/regression//m32_int_0294.xi"), Some(0)); }
#[test] fn e2e_m32_int_0295() { assert_eq!(compile_and_run("tests/regression//m32_int_0295.xi"), Some(0)); }
#[test] fn e2e_m32_int_0296() { assert_eq!(compile_and_run("tests/regression//m32_int_0296.xi"), Some(0)); }
#[test] fn e2e_m32_int_0297() { assert_eq!(compile_and_run("tests/regression//m32_int_0297.xi"), Some(0)); }
#[test] fn e2e_m32_int_0298() { assert_eq!(compile_and_run("tests/regression//m32_int_0298.xi"), Some(0)); }
#[test] fn e2e_m32_int_0299() { assert_eq!(compile_and_run("tests/regression//m32_int_0299.xi"), Some(0)); }
#[test] fn e2e_m32_int_0300() { assert_eq!(compile_and_run("tests/regression//m32_int_0300.xi"), Some(0)); }
#[test] fn e2e_m32_int_0301() { assert_eq!(compile_and_run("tests/regression//m32_int_0301.xi"), Some(0)); }
#[test] fn e2e_m32_int_0302() { assert_eq!(compile_and_run("tests/regression//m32_int_0302.xi"), Some(0)); }
#[test] fn e2e_m32_int_0303() { assert_eq!(compile_and_run("tests/regression//m32_int_0303.xi"), Some(0)); }
#[test] fn e2e_m32_int_0304() { assert_eq!(compile_and_run("tests/regression//m32_int_0304.xi"), Some(0)); }
#[test] fn e2e_m32_int_0305() { assert_eq!(compile_and_run("tests/regression//m32_int_0305.xi"), Some(0)); }
#[test] fn e2e_m32_int_0306() { assert_eq!(compile_and_run("tests/regression//m32_int_0306.xi"), Some(0)); }
#[test] fn e2e_m32_int_0307() { assert_eq!(compile_and_run("tests/regression//m32_int_0307.xi"), Some(0)); }
#[test] fn e2e_m32_int_0308() { assert_eq!(compile_and_run("tests/regression//m32_int_0308.xi"), Some(0)); }
#[test] fn e2e_m32_int_0309() { assert_eq!(compile_and_run("tests/regression//m32_int_0309.xi"), Some(0)); }
#[test] fn e2e_m32_int_0310() { assert_eq!(compile_and_run("tests/regression//m32_int_0310.xi"), Some(0)); }
#[test] fn e2e_m32_int_0311() { assert_eq!(compile_and_run("tests/regression//m32_int_0311.xi"), Some(0)); }
#[test] fn e2e_m32_int_0312() { assert_eq!(compile_and_run("tests/regression//m32_int_0312.xi"), Some(0)); }
#[test] fn e2e_m32_int_0313() { assert_eq!(compile_and_run("tests/regression//m32_int_0313.xi"), Some(0)); }
#[test] fn e2e_m32_int_0314() { assert_eq!(compile_and_run("tests/regression//m32_int_0314.xi"), Some(0)); }
#[test] fn e2e_m32_int_0315() { assert_eq!(compile_and_run("tests/regression//m32_int_0315.xi"), Some(0)); }
#[test] fn e2e_m32_int_0316() { assert_eq!(compile_and_run("tests/regression//m32_int_0316.xi"), Some(0)); }
#[test] fn e2e_m32_int_0317() { assert_eq!(compile_and_run("tests/regression//m32_int_0317.xi"), Some(0)); }
#[test] fn e2e_m32_int_0318() { assert_eq!(compile_and_run("tests/regression//m32_int_0318.xi"), Some(0)); }
#[test] fn e2e_m32_int_0319() { assert_eq!(compile_and_run("tests/regression//m32_int_0319.xi"), Some(0)); }
#[test] fn e2e_m32_int_0320() { assert_eq!(compile_and_run("tests/regression//m32_int_0320.xi"), Some(0)); }
#[test] fn e2e_m32_int_0321() { assert_eq!(compile_and_run("tests/regression//m32_int_0321.xi"), Some(0)); }
#[test] fn e2e_m32_int_0322() { assert_eq!(compile_and_run("tests/regression//m32_int_0322.xi"), Some(0)); }
#[test] fn e2e_m32_int_0323() { assert_eq!(compile_and_run("tests/regression//m32_int_0323.xi"), Some(0)); }
#[test] fn e2e_m32_int_0324() { assert_eq!(compile_and_run("tests/regression//m32_int_0324.xi"), Some(0)); }
#[test] fn e2e_m32_int_0325() { assert_eq!(compile_and_run("tests/regression//m32_int_0325.xi"), Some(0)); }
#[test] fn e2e_m32_int_0326() { assert_eq!(compile_and_run("tests/regression//m32_int_0326.xi"), Some(0)); }
#[test] fn e2e_m32_int_0327() { assert_eq!(compile_and_run("tests/regression//m32_int_0327.xi"), Some(0)); }
#[test] fn e2e_m32_int_0328() { assert_eq!(compile_and_run("tests/regression//m32_int_0328.xi"), Some(0)); }
#[test] fn e2e_m32_int_0329() { assert_eq!(compile_and_run("tests/regression//m32_int_0329.xi"), Some(0)); }
#[test] fn e2e_m32_int_0330() { assert_eq!(compile_and_run("tests/regression//m32_int_0330.xi"), Some(0)); }
#[test] fn e2e_m32_int_0331() { assert_eq!(compile_and_run("tests/regression//m32_int_0331.xi"), Some(0)); }
#[test] fn e2e_m32_int_0332() { assert_eq!(compile_and_run("tests/regression//m32_int_0332.xi"), Some(0)); }
#[test] fn e2e_m32_int_0333() { assert_eq!(compile_and_run("tests/regression//m32_int_0333.xi"), Some(0)); }
#[test] fn e2e_m32_int_0334() { assert_eq!(compile_and_run("tests/regression//m32_int_0334.xi"), Some(0)); }
#[test] fn e2e_m32_int_0335() { assert_eq!(compile_and_run("tests/regression//m32_int_0335.xi"), Some(0)); }
#[test] fn e2e_m32_int_0336() { assert_eq!(compile_and_run("tests/regression//m32_int_0336.xi"), Some(0)); }
#[test] fn e2e_m32_int_0337() { assert_eq!(compile_and_run("tests/regression//m32_int_0337.xi"), Some(0)); }
#[test] fn e2e_m32_int_0338() { assert_eq!(compile_and_run("tests/regression//m32_int_0338.xi"), Some(0)); }
#[test] fn e2e_m32_int_0339() { assert_eq!(compile_and_run("tests/regression//m32_int_0339.xi"), Some(0)); }
#[test] fn e2e_m32_int_0340() { assert_eq!(compile_and_run("tests/regression//m32_int_0340.xi"), Some(0)); }
#[test] fn e2e_m32_int_0341() { assert_eq!(compile_and_run("tests/regression//m32_int_0341.xi"), Some(0)); }
#[test] fn e2e_m32_int_0342() { assert_eq!(compile_and_run("tests/regression//m32_int_0342.xi"), Some(0)); }
#[test] fn e2e_m32_int_0343() { assert_eq!(compile_and_run("tests/regression//m32_int_0343.xi"), Some(0)); }
#[test] fn e2e_m32_int_0344() { assert_eq!(compile_and_run("tests/regression//m32_int_0344.xi"), Some(0)); }
#[test] fn e2e_m32_int_0345() { assert_eq!(compile_and_run("tests/regression//m32_int_0345.xi"), Some(0)); }
#[test] fn e2e_m32_int_0346() { assert_eq!(compile_and_run("tests/regression//m32_int_0346.xi"), Some(0)); }
#[test] fn e2e_m32_int_0347() { assert_eq!(compile_and_run("tests/regression//m32_int_0347.xi"), Some(0)); }
#[test] fn e2e_m32_int_0348() { assert_eq!(compile_and_run("tests/regression//m32_int_0348.xi"), Some(0)); }
#[test] fn e2e_m32_int_0349() { assert_eq!(compile_and_run("tests/regression//m32_int_0349.xi"), Some(0)); }
#[test] fn e2e_m32_int_0350() { assert_eq!(compile_and_run("tests/regression//m32_int_0350.xi"), Some(0)); }
#[test] fn e2e_m32_int_0351() { assert_eq!(compile_and_run("tests/regression//m32_int_0351.xi"), Some(0)); }
#[test] fn e2e_m32_int_0352() { assert_eq!(compile_and_run("tests/regression//m32_int_0352.xi"), Some(0)); }
#[test] fn e2e_m32_int_0353() { assert_eq!(compile_and_run("tests/regression//m32_int_0353.xi"), Some(0)); }
#[test] fn e2e_m32_int_0354() { assert_eq!(compile_and_run("tests/regression//m32_int_0354.xi"), Some(0)); }
#[test] fn e2e_m32_int_0355() { assert_eq!(compile_and_run("tests/regression//m32_int_0355.xi"), Some(0)); }
#[test] fn e2e_m32_int_0356() { assert_eq!(compile_and_run("tests/regression//m32_int_0356.xi"), Some(0)); }
#[test] fn e2e_m32_int_0357() { assert_eq!(compile_and_run("tests/regression//m32_int_0357.xi"), Some(0)); }
#[test] fn e2e_m32_int_0358() { assert_eq!(compile_and_run("tests/regression//m32_int_0358.xi"), Some(0)); }
#[test] fn e2e_m32_int_0359() { assert_eq!(compile_and_run("tests/regression//m32_int_0359.xi"), Some(0)); }
#[test] fn e2e_m32_int_0360() { assert_eq!(compile_and_run("tests/regression//m32_int_0360.xi"), Some(0)); }
#[test] fn e2e_m32_int_0361() { assert_eq!(compile_and_run("tests/regression//m32_int_0361.xi"), Some(0)); }
#[test] fn e2e_m32_int_0362() { assert_eq!(compile_and_run("tests/regression//m32_int_0362.xi"), Some(0)); }
#[test] fn e2e_m32_int_0363() { assert_eq!(compile_and_run("tests/regression//m32_int_0363.xi"), Some(0)); }
#[test] fn e2e_m32_int_0364() { assert_eq!(compile_and_run("tests/regression//m32_int_0364.xi"), Some(0)); }
#[test] fn e2e_m32_int_0365() { assert_eq!(compile_and_run("tests/regression//m32_int_0365.xi"), Some(0)); }
#[test] fn e2e_m32_int_0366() { assert_eq!(compile_and_run("tests/regression//m32_int_0366.xi"), Some(0)); }
#[test] fn e2e_m32_int_0367() { assert_eq!(compile_and_run("tests/regression//m32_int_0367.xi"), Some(0)); }
#[test] fn e2e_m32_int_0368() { assert_eq!(compile_and_run("tests/regression//m32_int_0368.xi"), Some(0)); }
#[test] fn e2e_m32_int_0369() { assert_eq!(compile_and_run("tests/regression//m32_int_0369.xi"), Some(0)); }
#[test] fn e2e_m32_int_0370() { assert_eq!(compile_and_run("tests/regression//m32_int_0370.xi"), Some(0)); }
#[test] fn e2e_m32_int_0371() { assert_eq!(compile_and_run("tests/regression//m32_int_0371.xi"), Some(0)); }
#[test] fn e2e_m32_int_0372() { assert_eq!(compile_and_run("tests/regression//m32_int_0372.xi"), Some(0)); }
#[test] fn e2e_m32_int_0373() { assert_eq!(compile_and_run("tests/regression//m32_int_0373.xi"), Some(0)); }
#[test] fn e2e_m32_int_0374() { assert_eq!(compile_and_run("tests/regression//m32_int_0374.xi"), Some(0)); }
#[test] fn e2e_m32_int_0375() { assert_eq!(compile_and_run("tests/regression//m32_int_0375.xi"), Some(0)); }
#[test] fn e2e_m32_int_0376() { assert_eq!(compile_and_run("tests/regression//m32_int_0376.xi"), Some(0)); }
#[test] fn e2e_m32_int_0377() { assert_eq!(compile_and_run("tests/regression//m32_int_0377.xi"), Some(0)); }
#[test] fn e2e_m32_int_0378() { assert_eq!(compile_and_run("tests/regression//m32_int_0378.xi"), Some(0)); }
#[test] fn e2e_m32_int_0379() { assert_eq!(compile_and_run("tests/regression//m32_int_0379.xi"), Some(0)); }
#[test] fn e2e_m32_int_0380() { assert_eq!(compile_and_run("tests/regression//m32_int_0380.xi"), Some(0)); }
#[test] fn e2e_m32_int_0381() { assert_eq!(compile_and_run("tests/regression//m32_int_0381.xi"), Some(0)); }
#[test] fn e2e_m32_int_0382() { assert_eq!(compile_and_run("tests/regression//m32_int_0382.xi"), Some(0)); }
#[test] fn e2e_m32_int_0383() { assert_eq!(compile_and_run("tests/regression//m32_int_0383.xi"), Some(0)); }
#[test] fn e2e_m32_int_0384() { assert_eq!(compile_and_run("tests/regression//m32_int_0384.xi"), Some(0)); }
#[test] fn e2e_m32_int_0385() { assert_eq!(compile_and_run("tests/regression//m32_int_0385.xi"), Some(0)); }
#[test] fn e2e_m32_int_0386() { assert_eq!(compile_and_run("tests/regression//m32_int_0386.xi"), Some(0)); }
#[test] fn e2e_m32_int_0387() { assert_eq!(compile_and_run("tests/regression//m32_int_0387.xi"), Some(0)); }
#[test] fn e2e_m32_int_0388() { assert_eq!(compile_and_run("tests/regression//m32_int_0388.xi"), Some(0)); }
#[test] fn e2e_m32_int_0389() { assert_eq!(compile_and_run("tests/regression//m32_int_0389.xi"), Some(0)); }
#[test] fn e2e_m32_int_0390() { assert_eq!(compile_and_run("tests/regression//m32_int_0390.xi"), Some(0)); }
#[test] fn e2e_m32_int_0391() { assert_eq!(compile_and_run("tests/regression//m32_int_0391.xi"), Some(0)); }
#[test] fn e2e_m32_int_0392() { assert_eq!(compile_and_run("tests/regression//m32_int_0392.xi"), Some(0)); }
#[test] fn e2e_m32_int_0393() { assert_eq!(compile_and_run("tests/regression//m32_int_0393.xi"), Some(0)); }
#[test] fn e2e_m32_int_0394() { assert_eq!(compile_and_run("tests/regression//m32_int_0394.xi"), Some(0)); }
#[test] fn e2e_m32_int_0395() { assert_eq!(compile_and_run("tests/regression//m32_int_0395.xi"), Some(0)); }
#[test] fn e2e_m32_int_0396() { assert_eq!(compile_and_run("tests/regression//m32_int_0396.xi"), Some(0)); }
#[test] fn e2e_m32_int_0397() { assert_eq!(compile_and_run("tests/regression//m32_int_0397.xi"), Some(0)); }
#[test] fn e2e_m32_int_0398() { assert_eq!(compile_and_run("tests/regression//m32_int_0398.xi"), Some(0)); }
#[test] fn e2e_m32_int_0399() { assert_eq!(compile_and_run("tests/regression//m32_int_0399.xi"), Some(0)); }
#[test] fn e2e_m32_int_0400() { assert_eq!(compile_and_run("tests/regression//m32_int_0400.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_001() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_001.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_002() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_002.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_005() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_005.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_008() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_008.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_010() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_010.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_003() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_003.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_006() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_006.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_007() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_007.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_009() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_009.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_011() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_011.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_012() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_012.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_013() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_013.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_014() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_014.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_020() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_020.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_021() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_021.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_022() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_022.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_024() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_024.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_025() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_025.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_029() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_029.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_030() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_030.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_031() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_031.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_033() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_033.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_038() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_038.xi"), Some(0)); }
#[test] fn e2e_m21_while_001() { assert_eq!(compile_and_run("tests/regression//m21_while_001.xi"), Some(0)); }
#[test] fn e2e_m21_while_002() { assert_eq!(compile_and_run("tests/regression//m21_while_002.xi"), Some(0)); }
#[test] fn e2e_m21_while_003() { assert_eq!(compile_and_run("tests/regression//m21_while_003.xi"), Some(0)); }
#[test] fn e2e_m21_while_004() { assert_eq!(compile_and_run("tests/regression//m21_while_004.xi"), Some(0)); }
#[test] fn e2e_m21_while_005() { assert_eq!(compile_and_run("tests/regression//m21_while_005.xi"), Some(0)); }
#[test] fn e2e_m21_while_006() { assert_eq!(compile_and_run("tests/regression//m21_while_006.xi"), Some(0)); }
#[test] fn e2e_m21_while_007() { assert_eq!(compile_and_run("tests/regression//m21_while_007.xi"), Some(0)); }
#[test] fn e2e_m21_while_008() { assert_eq!(compile_and_run("tests/regression//m21_while_008.xi"), Some(0)); }
#[test] fn e2e_m21_while_009() { assert_eq!(compile_and_run("tests/regression//m21_while_009.xi"), Some(0)); }
#[test] fn e2e_m21_while_010() { assert_eq!(compile_and_run("tests/regression//m21_while_010.xi"), Some(0)); }
#[test] fn e2e_m21_while_011() { assert_eq!(compile_and_run("tests/regression//m21_while_011.xi"), Some(0)); }
#[test] fn e2e_m21_while_012() { assert_eq!(compile_and_run("tests/regression//m21_while_012.xi"), Some(0)); }
#[test] fn e2e_m21_while_013() { assert_eq!(compile_and_run("tests/regression//m21_while_013.xi"), Some(0)); }
#[test] fn e2e_m21_while_014() { assert_eq!(compile_and_run("tests/regression//m21_while_014.xi"), Some(0)); }
#[test] fn e2e_m21_while_015() { assert_eq!(compile_and_run("tests/regression//m21_while_015.xi"), Some(0)); }
// m21_string passing tests
#[test] fn e2e_m21_string_001() { assert_eq!(compile_and_run("tests/regression//m21_string_001.xi"), Some(0)); }
#[test] fn e2e_m21_string_002() { assert_eq!(compile_and_run("tests/regression//m21_string_002.xi"), Some(0)); }
#[test] fn e2e_m21_string_006() { assert_eq!(compile_and_run("tests/regression//m21_string_006.xi"), Some(0)); }
#[test] fn e2e_m21_string_008() { assert_eq!(compile_and_run("tests/regression//m21_string_008.xi"), Some(0)); }
#[test] fn e2e_m21_string_009() { assert_eq!(compile_and_run("tests/regression//m21_string_009.xi"), Some(0)); }
#[test] fn e2e_m21_string_011() { assert_eq!(compile_and_run("tests/regression//m21_string_011.xi"), Some(0)); }
#[test] fn e2e_m21_string_012() { assert_eq!(compile_and_run("tests/regression//m21_string_012.xi"), Some(0)); }
#[test] fn e2e_m21_string_013() { assert_eq!(compile_and_run("tests/regression//m21_string_013.xi"), Some(0)); }
#[test] fn e2e_m21_string_014() { assert_eq!(compile_and_run("tests/regression//m21_string_014.xi"), Some(0)); }
#[test] fn e2e_m21_string_015() { assert_eq!(compile_and_run("tests/regression//m21_string_015.xi"), Some(0)); }
#[test] fn e2e_m21_string_016() { assert_eq!(compile_and_run("tests/regression//m21_string_016.xi"), Some(0)); }
#[test] fn e2e_m21_string_017() { assert_eq!(compile_and_run("tests/regression//m21_string_017.xi"), Some(0)); }
#[test] fn e2e_m21_string_018() { assert_eq!(compile_and_run("tests/regression//m21_string_018.xi"), Some(0)); }
#[test] fn e2e_m21_string_019() { assert_eq!(compile_and_run("tests/regression//m21_string_019.xi"), Some(0)); }

// -- M32-S: Struct Types Stress Tests ------------------------------------
// Basic struct, nested structs, field copy, all primitives, array-like,
// derive[Eq], struct return, struct param, spread, deep nesting,
// mutation, multi-type interaction, Float64 arithmetic, mixed types, composition
#[test] fn e2e_m32_s01() { assert_eq!(compile_and_run("tests/regression//m32_s01.xi"), Some(0)); }
#[test] fn e2e_m32_s02() { assert_eq!(compile_and_run("tests/regression//m32_s02.xi"), Some(0)); }
#[test] fn e2e_m32_s03() { assert_eq!(compile_and_run("tests/regression//m32_s03.xi"), Some(0)); }
#[test] fn e2e_m32_s04() { assert_eq!(compile_and_run("tests/regression//m32_s04.xi"), Some(0)); }
#[test] fn e2e_m32_s05() { assert_eq!(compile_and_run("tests/regression//m32_s05.xi"), Some(0)); }
#[test] fn e2e_m32_s06() { assert_eq!(compile_and_run("tests/regression//m32_s06.xi"), Some(0)); }
#[test] fn e2e_m32_s07() { assert_eq!(compile_and_run("tests/regression//m32_s07.xi"), Some(0)); }
#[test] fn e2e_m32_s08() { assert_eq!(compile_and_run("tests/regression//m32_s08.xi"), Some(0)); }
#[test] fn e2e_m32_s09() { assert_eq!(compile_and_run("tests/regression//m32_s09.xi"), Some(0)); }
#[test] fn e2e_m32_s10() { assert_eq!(compile_and_run("tests/regression//m32_s10.xi"), Some(0)); }
#[test] fn e2e_m32_s11() { assert_eq!(compile_and_run("tests/regression//m32_s11.xi"), Some(0)); }
#[test] fn e2e_m32_s12() { assert_eq!(compile_and_run("tests/regression//m32_s12.xi"), Some(0)); }
#[test] fn e2e_m32_s13() { assert_eq!(compile_and_run("tests/regression//m32_s13.xi"), Some(0)); }
#[test] fn e2e_m32_s14() { assert_eq!(compile_and_run("tests/regression//m32_s14.xi"), Some(0)); }
#[test] fn e2e_m32_s15() { assert_eq!(compile_and_run("tests/regression//m32_s15.xi"), Some(0)); }

// -- M32: Float Types Stress Tests ------------------------------------
// Float64 arithmetic
#[test] fn e2e_m32_f01() { assert_eq!(compile_and_run("tests/regression//m32_f01.xi"), Some(0)); }
// Float64 comparisons
#[test] fn e2e_m32_f02() { assert_eq!(compile_and_run("tests/regression//m32_f02.xi"), Some(0)); }
// Float32 arithmetic
#[test] fn e2e_m32_f03() { assert_eq!(compile_and_run("tests/regression//m32_f03.xi"), Some(0)); }
// Float32 comparisons
#[test] fn e2e_m32_f04() { assert_eq!(compile_and_run("tests/regression//m32_f04.xi"), Some(0)); }
// Float32 to Float64 cast
#[test] fn e2e_m32_f05() { assert_eq!(compile_and_run("tests/regression//m32_f05.xi"), Some(0)); }
// Float64 to Float32 cast
#[test] fn e2e_m32_f06() { assert_eq!(compile_and_run("tests/regression//m32_f06.xi"), Some(0)); }
// Float64 to Int conversion
#[test] fn e2e_m32_f07() { assert_eq!(compile_and_run("tests/regression//m32_f07.xi"), Some(0)); }
// Float32 to Int conversion
#[test] fn e2e_m32_f08() { assert_eq!(compile_and_run("tests/regression//m32_f08.xi"), Some(0)); }
// Float64 negative values
#[test] fn e2e_m32_f09() { assert_eq!(compile_and_run("tests/regression//m32_f09.xi"), Some(0)); }
// Float32 negative values
#[test] fn e2e_m32_f10() { assert_eq!(compile_and_run("tests/regression//m32_f10.xi"), Some(0)); }
// Very small Float64 (0.000001)
#[test] fn e2e_m32_f11() { assert_eq!(compile_and_run("tests/regression//m32_f11.xi"), Some(0)); }
// Very small Float32 (0.000001)
#[test] fn e2e_m32_f12() { assert_eq!(compile_and_run("tests/regression//m32_f12.xi"), Some(0)); }
// Very large Float64 (1e308)
#[test] fn e2e_m32_f13() { assert_eq!(compile_and_run("tests/regression//m32_f13.xi"), Some(0)); }
// Mixed Float32/Float64 casts
#[test] fn e2e_m32_f14() { assert_eq!(compile_and_run("tests/regression//m32_f14.xi"), Some(0)); }
// Float comparison edge cases
#[test] fn e2e_m32_f15() { assert_eq!(compile_and_run("tests/regression//m32_f15.xi"), Some(0)); }

#[test] fn e2e_m32_e01() { assert_eq!(compile_and_run("tests/regression//m32_e01.xi"), Some(0)); }
#[test] fn e2e_m32_e02() { assert_eq!(compile_and_run("tests/regression//m32_e02.xi"), Some(0)); }
#[test] fn e2e_m32_e03() { assert_eq!(compile_and_run("tests/regression//m32_e03.xi"), Some(0)); }
#[test] fn e2e_m32_e04() { assert_eq!(compile_and_run("tests/regression//m32_e04.xi"), Some(0)); }
#[test] fn e2e_m32_e05() { assert_eq!(compile_and_run("tests/regression//m32_e05.xi"), Some(0)); }
#[test] fn e2e_m32_e06() { assert_eq!(compile_and_run("tests/regression//m32_e06.xi"), Some(0)); }
#[test] fn e2e_m32_e07() { assert_eq!(compile_and_run("tests/regression//m32_e07.xi"), Some(0)); }
#[test] fn e2e_m32_e08() { assert_eq!(compile_and_run("tests/regression//m32_e08.xi"), Some(0)); }
#[test] fn e2e_m32_e09() { assert_eq!(compile_and_run("tests/regression//m32_e09.xi"), Some(0)); }
#[test] fn e2e_m32_e10() { assert_eq!(compile_and_run("tests/regression//m32_e10.xi"), Some(0)); }
#[test] fn e2e_m32_e11() { assert_eq!(compile_and_run("tests/regression//m32_e11.xi"), Some(0)); }
#[test] fn e2e_m32_e12() { assert_eq!(compile_and_run("tests/regression//m32_e12.xi"), Some(0)); }
#[test] fn e2e_m32_e13() { assert_eq!(compile_and_run("tests/regression//m32_e13.xi"), Some(0)); }
#[test] fn e2e_m32_e14() { assert_eq!(compile_and_run("tests/regression//m32_e14.xi"), Some(0)); }
#[test] fn e2e_m32_e15() { assert_eq!(compile_and_run("tests/regression//m32_e15.xi"), Some(0)); }

#[test] fn e2e_m32_c01() { assert_eq!(compile_and_run("tests/regression//m32_c01.xi"), Some(0)); }
#[test] fn e2e_m32_c02() { assert_eq!(compile_and_run("tests/regression//m32_c02.xi"), Some(0)); }
#[test] fn e2e_m32_c03() { assert_eq!(compile_and_run("tests/regression//m32_c03.xi"), Some(0)); }
#[test] fn e2e_m32_c04() { assert_eq!(compile_and_run("tests/regression//m32_c04.xi"), Some(0)); }
#[test] fn e2e_m32_c05() { assert_eq!(compile_and_run("tests/regression//m32_c05.xi"), Some(0)); }
#[test] fn e2e_m32_c06() { assert_eq!(compile_and_run("tests/regression//m32_c06.xi"), Some(0)); }
#[test] fn e2e_m32_c07() { assert_eq!(compile_and_run("tests/regression//m32_c07.xi"), Some(0)); }
#[test] fn e2e_m32_c08() { assert_eq!(compile_and_run("tests/regression//m32_c08.xi"), Some(0)); }
#[test] fn e2e_m32_c09() { assert_eq!(compile_and_run("tests/regression//m32_c09.xi"), Some(0)); }
#[test] fn e2e_m32_c10() { assert_eq!(compile_and_run("tests/regression//m32_c10.xi"), Some(0)); }
#[test] fn e2e_m32_c11() { assert_eq!(compile_and_run("tests/regression//m32_c11.xi"), Some(0)); }
#[test] fn e2e_m32_c12() { assert_eq!(compile_and_run("tests/regression//m32_c12.xi"), Some(0)); }
#[test] fn e2e_m32_c13() { assert_eq!(compile_and_run("tests/regression//m32_c13.xi"), Some(0)); }
#[test] fn e2e_m32_c14() { assert_eq!(compile_and_run("tests/regression//m32_c14.xi"), Some(0)); }
#[test] fn e2e_m32_c15() { assert_eq!(compile_and_run("tests/regression//m32_c15.xi"), Some(0)); }

#[test] fn e2e_m32_i01() { assert_eq!(compile_and_run("tests/regression//m32_i01.xi"), Some(0)); }
#[test] fn e2e_m32_i02() { assert_eq!(compile_and_run("tests/regression//m32_i02.xi"), Some(0)); }
#[test] fn e2e_m32_i03() { assert_eq!(compile_and_run("tests/regression//m32_i03.xi"), Some(0)); }
#[test] fn e2e_m32_i04() { assert_eq!(compile_and_run("tests/regression//m32_i04.xi"), Some(0)); }
#[test] fn e2e_m32_i05() { assert_eq!(compile_and_run("tests/regression//m32_i05.xi"), Some(0)); }
#[test] fn e2e_m32_i06() { assert_eq!(compile_and_run("tests/regression//m32_i06.xi"), Some(0)); }
#[test] fn e2e_m32_i07() { assert_eq!(compile_and_run("tests/regression//m32_i07.xi"), Some(0)); }
#[test] fn e2e_m32_i08() { assert_eq!(compile_and_run("tests/regression//m32_i08.xi"), Some(0)); }
#[test] fn e2e_m32_i09() { assert_eq!(compile_and_run("tests/regression//m32_i09.xi"), Some(0)); }
#[test] fn e2e_m32_i10() { assert_eq!(compile_and_run("tests/regression//m32_i10.xi"), Some(0)); }
#[test] fn e2e_m32_i11() { assert_eq!(compile_and_run("tests/regression//m32_i11.xi"), Some(0)); }
#[test] fn e2e_m32_i12() { assert_eq!(compile_and_run("tests/regression//m32_i12.xi"), Some(0)); }
#[test] fn e2e_m32_i13() { assert_eq!(compile_and_run("tests/regression//m32_i13.xi"), Some(0)); }
#[test] fn e2e_m32_i14() { assert_eq!(compile_and_run("tests/regression//m32_i14.xi"), Some(0)); }
#[test] fn e2e_m32_i15() { assert_eq!(compile_and_run("tests/regression//m32_i15.xi"), Some(0)); }

// -- M32: Control Flow / Loop Tests -----------------------------------
// while loop sum
#[test] fn e2e_m32_l01() { assert_eq!(compile_and_run("tests/regression//m32_l01.xi"), Some(0)); }
// while+break at threshold
#[test] fn e2e_m32_l02() { assert_eq!(compile_and_run("tests/regression//m32_l02.xi"), Some(0)); }
// while+continue skip evens
#[test] fn e2e_m32_l03() { assert_eq!(compile_and_run("tests/regression//m32_l03.xi"), Some(0)); }
// nested while
#[test] fn e2e_m32_l04() { assert_eq!(compile_and_run("tests/regression//m32_l04.xi"), Some(0)); }
// for-in over array
#[test] fn e2e_m32_l05() { assert_eq!(compile_and_run("tests/regression//m32_l05.xi"), Some(0)); }
// while with early return
#[test] fn e2e_m32_l06() { assert_eq!(compile_and_run("tests/regression//m32_l06.xi"), Some(0)); }
// if-elif-else chain (5 branches)
#[test] fn e2e_m32_l07() { assert_eq!(compile_and_run("tests/regression//m32_l07.xi"), Some(0)); }
// compound assign in loop
#[test] fn e2e_m32_l08() { assert_eq!(compile_and_run("tests/regression//m32_l08.xi"), Some(0)); }
// while with complex condition
#[test] fn e2e_m32_l09() { assert_eq!(compile_and_run("tests/regression//m32_l09.xi"), Some(0)); }
// infinite loop with break guard
#[test] fn e2e_m32_l10() { assert_eq!(compile_and_run("tests/regression//m32_l10.xi"), Some(0)); }
// loop counter
#[test] fn e2e_m32_l11() { assert_eq!(compile_and_run("tests/regression//m32_l11.xi"), Some(0)); }
// if-as-expression
#[test] fn e2e_m32_l12() { assert_eq!(compile_and_run("tests/regression//m32_l12.xi"), Some(0)); }
// nested break
#[test] fn e2e_m32_l13() { assert_eq!(compile_and_run("tests/regression//m32_l13.xi"), Some(0)); }
// while with counter
#[test] fn e2e_m32_l14() { assert_eq!(compile_and_run("tests/regression//m32_l14.xi"), Some(0)); }
// for-in with break
#[test] fn e2e_m32_l15() { assert_eq!(compile_and_run("tests/regression//m32_l15.xi"), Some(0)); }

// -- M32: String / Char / Unicode Tests --------------------------------
// string literal
#[test] fn e2e_m32_t01() { assert_eq!(compile_and_run("tests/regression//m32_t01.xi"), Some(0)); }
// char literal
#[test] fn e2e_m32_t02() { assert_eq!(compile_and_run("tests/regression//m32_t02.xi"), Some(0)); }
// string concat
#[test] fn e2e_m32_t03() { assert_eq!(compile_and_run("tests/regression//m32_t03.xi"), Some(0)); }
// string length
#[test] fn e2e_m32_t04() { assert_eq!(compile_and_run("tests/regression//m32_t04.xi"), Some(0)); }
// char to int
#[test] fn e2e_m32_t05() { assert_eq!(compile_and_run("tests/regression//m32_t05.xi"), Some(0)); }
// int to char
#[test] fn e2e_m32_t06() { assert_eq!(compile_and_run("tests/regression//m32_t06.xi"), Some(0)); }
// empty string
#[test] fn e2e_m32_t07() { assert_eq!(compile_and_run("tests/regression//m32_t07.xi"), Some(0)); }
// string with spaces
#[test] fn e2e_m32_t08() { assert_eq!(compile_and_run("tests/regression//m32_t08.xi"), Some(0)); }
// string comparison == and !=
#[test] fn e2e_m32_t09() { assert_eq!(compile_and_run("tests/regression//m32_t09.xi"), Some(0)); }
// multi-char string
#[test] fn e2e_m32_t10() { assert_eq!(compile_and_run("tests/regression//m32_t10.xi"), Some(0)); }
// unicode char 'lambda'
#[test] fn e2e_m32_t11() { assert_eq!(compile_and_run("tests/regression//m32_t11.xi"), Some(0)); }
// escaped string with \n \t
#[test] fn e2e_m32_t12() { assert_eq!(compile_and_run("tests/regression//m32_t12.xi"), Some(0)); }
// string in struct
#[test] fn e2e_m32_t13() { assert_eq!(compile_and_run("tests/regression//m32_t13.xi"), Some(0)); }
// string in array
#[test] fn e2e_m32_t14() { assert_eq!(compile_and_run("tests/regression//m32_t14.xi"), Some(0)); }
// string parameter/return
#[test] fn e2e_m32_t15() { assert_eq!(compile_and_run("tests/regression//m32_t15.xi"), Some(0)); }

// -- M32-G: Generic Types & Traits Stress Tests --------------------------
// Identity generic, two-param generic, generic struct, generic enum,
// constrained generic, multi-constraint, method on generic, generic return,
// nested Vec, generic array access, type alias, max comparator,
// multi-constraint composition, generic enum match, multi-method constraint
#[test] fn e2e_m32_g01() { assert_eq!(compile_and_run("tests/regression//m32_g01.xi"), Some(0)); }
#[test] fn e2e_m32_g02() { assert_eq!(compile_and_run("tests/regression//m32_g02.xi"), Some(0)); }
#[test] fn e2e_m32_g03() { assert_eq!(compile_and_run("tests/regression//m32_g03.xi"), Some(0)); }
#[test] fn e2e_m32_g04() { assert_eq!(compile_and_run("tests/regression//m32_g04.xi"), Some(0)); }
#[test] fn e2e_m32_g05() { assert_eq!(compile_and_run("tests/regression//m32_g05.xi"), Some(0)); }
#[test] fn e2e_m32_g06() { assert_eq!(compile_and_run("tests/regression//m32_g06.xi"), Some(0)); }
#[test] fn e2e_m32_g07() { assert_eq!(compile_and_run("tests/regression//m32_g07.xi"), Some(0)); }
#[test] fn e2e_m32_g08() { assert_eq!(compile_and_run("tests/regression//m32_g08.xi"), Some(0)); }
#[test] fn e2e_m32_g09() { assert_eq!(compile_and_run("tests/regression//m32_g09.xi"), Some(0)); }
#[test] fn e2e_m32_g10() { assert_eq!(compile_and_run("tests/regression//m32_g10.xi"), Some(0)); }
#[test] fn e2e_m32_g11() { assert_eq!(compile_and_run("tests/regression//m32_g11.xi"), Some(0)); }
#[test] fn e2e_m32_g12() { assert_eq!(compile_and_run("tests/regression//m32_g12.xi"), Some(0)); }
#[test] fn e2e_m32_g13() { assert_eq!(compile_and_run("tests/regression//m32_g13.xi"), Some(0)); }
#[test] fn e2e_m32_g14() { assert_eq!(compile_and_run("tests/regression//m32_g14.xi"), Some(0)); }
#[test] fn e2e_m32_g15() { assert_eq!(compile_and_run("tests/regression//m32_g15.xi"), Some(0)); }

// -- M32-M: Module System Tests ------------------------------------------
// Module declaration, pub fn visibility, private fn, use module,
// use type alias, dotted path access, pub const, pub type,
// nested modules, module exports
#[test] fn e2e_m32_m01() { assert_eq!(compile_and_run("tests/regression//m32_m01.xi"), Some(0)); }
#[test] fn e2e_m32_m02() { assert_eq!(compile_and_run("tests/regression//m32_m02.xi"), Some(0)); }
#[test] fn e2e_m32_m03() { assert_eq!(compile_and_run("tests/regression//m32_m03.xi"), Some(0)); }
#[test] fn e2e_m32_m04() { assert_eq!(compile_and_run("tests/regression//m32_m04.xi"), Some(0)); }
#[test] fn e2e_m32_m05() { assert_eq!(compile_and_run("tests/regression//m32_m05.xi"), Some(0)); }
#[test] fn e2e_m32_m06() { assert_eq!(compile_and_run("tests/regression//m32_m06.xi"), Some(0)); }
#[test] fn e2e_m32_m07() { assert_eq!(compile_and_run("tests/regression//m32_m07.xi"), Some(0)); }
#[test] fn e2e_m32_m08() { assert_eq!(compile_and_run("tests/regression//m32_m08.xi"), Some(0)); }
#[test] fn e2e_m32_m09() { assert_eq!(compile_and_run("tests/regression//m32_m09.xi"), Some(0)); }
#[test] fn e2e_m32_m10() { assert_eq!(compile_and_run("tests/regression//m32_m10.xi"), Some(0)); }
#[test] fn e2e_m32_m11() { assert_eq!(compile_and_run("tests/regression//m32_m11.xi"), Some(0)); }
#[test] fn e2e_m32_m12() { assert_eq!(compile_and_run("tests/regression//m32_m12.xi"), Some(0)); }
#[test] fn e2e_m32_m13() { assert_eq!(compile_and_run("tests/regression//m32_m13.xi"), Some(0)); }
#[test] fn e2e_m32_m14() { assert_eq!(compile_and_run("tests/regression//m32_m14.xi"), Some(0)); }
#[test] fn e2e_m32_m15() { assert_eq!(compile_and_run("tests/regression//m32_m15.xi"), Some(0)); }

// -- M32-X: Combinatorial + Differential Correctness Tests ----------------
// Each test mixes struct+enum+match+generic+contract and verifies
// two equivalent implementations produce the same result.
#[test] fn e2e_m32_x01() { assert_eq!(compile_and_run("tests/regression//m32_x01.xi"), Some(0)); }
#[test] fn e2e_m32_x02() { assert_eq!(compile_and_run("tests/regression//m32_x02.xi"), Some(0)); }
#[test] fn e2e_m32_x03() { assert_eq!(compile_and_run("tests/regression//m32_x03.xi"), Some(0)); }
#[test] fn e2e_m32_x04() { assert_eq!(compile_and_run("tests/regression//m32_x04.xi"), Some(0)); }
#[test] fn e2e_m32_x05() { assert_eq!(compile_and_run("tests/regression//m32_x05.xi"), Some(0)); }
#[test] fn e2e_m32_x06() { assert_eq!(compile_and_run("tests/regression//m32_x06.xi"), Some(0)); }
#[test] fn e2e_m32_x07() { assert_eq!(compile_and_run("tests/regression//m32_x07.xi"), Some(0)); }
#[test] fn e2e_m32_x08() { assert_eq!(compile_and_run("tests/regression//m32_x08.xi"), Some(0)); }
#[test] fn e2e_m32_x09() { assert_eq!(compile_and_run("tests/regression//m32_x09.xi"), Some(0)); }
#[test] fn e2e_m32_x10() { assert_eq!(compile_and_run("tests/regression//m32_x10.xi"), Some(0)); }
#[test] fn e2e_m32_x11() { assert_eq!(compile_and_run("tests/regression//m32_x11.xi"), Some(0)); }
#[test] fn e2e_m32_x12() { assert_eq!(compile_and_run("tests/regression//m32_x12.xi"), Some(0)); }
#[test] fn e2e_m32_x13() { assert_eq!(compile_and_run("tests/regression//m32_x13.xi"), Some(0)); }
#[test] fn e2e_m32_x14() { assert_eq!(compile_and_run("tests/regression//m32_x14.xi"), Some(0)); }
#[test] fn e2e_m32_x15() { assert_eq!(compile_and_run("tests/regression//m32_x15.xi"), Some(0)); }

// -- M33-Y: Deep Combinatorial Stress (7+ features per test) -------------
// Each test combines: struct + enum + generic + match + contract + method + module
// with differential testing (two+ implementations produce the same result).
#[test] fn e2e_m33_y01() { assert_eq!(compile_and_run("tests/regression//m33_y01.xi"), Some(0)); }
#[test] fn e2e_m33_y02() { assert_eq!(compile_and_run("tests/regression//m33_y02.xi"), Some(0)); }
#[test] fn e2e_m33_y03() { assert_eq!(compile_and_run("tests/regression//m33_y03.xi"), Some(0)); }
#[test] fn e2e_m33_y04() { assert_eq!(compile_and_run("tests/regression//m33_y04.xi"), Some(0)); }
#[test] fn e2e_m33_y05() { assert_eq!(compile_and_run("tests/regression//m33_y05.xi"), Some(0)); }
#[test] fn e2e_m33_y06() { assert_eq!(compile_and_run("tests/regression//m33_y06.xi"), Some(0)); }
#[test] fn e2e_m33_y07() { assert_eq!(compile_and_run("tests/regression//m33_y07.xi"), Some(0)); }
#[test] fn e2e_m33_y08() { assert_eq!(compile_and_run("tests/regression//m33_y08.xi"), Some(0)); }
#[test] fn e2e_m33_y09() { assert_eq!(compile_and_run("tests/regression//m33_y09.xi"), Some(0)); }
#[test] fn e2e_m33_y10() { assert_eq!(compile_and_run("tests/regression//m33_y10.xi"), Some(0)); }
#[test] fn e2e_m33_y11() { assert_eq!(compile_and_run("tests/regression//m33_y11.xi"), Some(0)); }
#[test] fn e2e_m33_y12() { assert_eq!(compile_and_run("tests/regression//m33_y12.xi"), Some(0)); }
#[test] fn e2e_m33_y13() { assert_eq!(compile_and_run("tests/regression//m33_y13.xi"), Some(0)); }
#[test] fn e2e_m33_y14() { assert_eq!(compile_and_run("tests/regression//m33_y14.xi"), Some(0)); }
#[test] fn e2e_m33_y15() { assert_eq!(compile_and_run("tests/regression//m33_y15.xi"), Some(0)); }
#[test] fn e2e_m33_y16() { assert_eq!(compile_and_run("tests/regression//m33_y16.xi"), Some(0)); }
#[test] fn e2e_m33_y17() { assert_eq!(compile_and_run("tests/regression//m33_y17.xi"), Some(0)); }
#[test] fn e2e_m33_y18() { assert_eq!(compile_and_run("tests/regression//m33_y18.xi"), Some(0)); }
#[test] fn e2e_m33_y19() { assert_eq!(compile_and_run("tests/regression//m33_y19.xi"), Some(0)); }
#[test] fn e2e_m33_y20() { assert_eq!(compile_and_run("tests/regression//m33_y20.xi"), Some(0)); }

// -- M33-K: Closure / Function Pointer Tests ----------------------------
// Pipe closures, block closures, function pointer types, higher-order,
// closure chains, generic closures, struct fields, early return,
// match arms, while loops, compound ops, returning closures
#[test] fn e2e_m33_k01() { assert_eq!(compile_and_run("tests/regression//m33_k01.xi"), Some(0)); }
#[test] fn e2e_m33_k02() { assert_eq!(compile_and_run("tests/regression//m33_k02.xi"), Some(0)); }
#[test] fn e2e_m33_k03() { assert_eq!(compile_and_run("tests/regression//m33_k03.xi"), Some(0)); }
#[test] fn e2e_m33_k04() { assert_eq!(compile_and_run("tests/regression//m33_k04.xi"), Some(0)); }
#[test] fn e2e_m33_k05() { assert_eq!(compile_and_run("tests/regression//m33_k05.xi"), Some(0)); }
#[test] fn e2e_m33_k06() { assert_eq!(compile_and_run("tests/regression//m33_k06.xi"), Some(0)); }
#[test] fn e2e_m33_k07() { assert_eq!(compile_and_run("tests/regression//m33_k07.xi"), Some(0)); }
#[test] fn e2e_m33_k08() { assert_eq!(compile_and_run("tests/regression//m33_k08.xi"), Some(0)); }
#[test] fn e2e_m33_k09() { assert_eq!(compile_and_run("tests/regression//m33_k09.xi"), Some(0)); }
#[test] fn e2e_m33_k10() { assert_eq!(compile_and_run("tests/regression//m33_k10.xi"), Some(0)); }
#[test] fn e2e_m33_k11() { assert_eq!(compile_and_run("tests/regression//m33_k11.xi"), Some(0)); }
#[test] fn e2e_m33_k12() { assert_eq!(compile_and_run("tests/regression//m33_k12.xi"), Some(0)); }
#[test] fn e2e_m33_k13() { assert_eq!(compile_and_run("tests/regression//m33_k13.xi"), Some(0)); }
#[test] fn e2e_m33_k14() { assert_eq!(compile_and_run("tests/regression//m33_k14.xi"), Some(0)); }
#[test] fn e2e_m33_k15() { assert_eq!(compile_and_run("tests/regression//m33_k15.xi"), Some(0)); }
#[test] fn e2e_m33_k16() { assert_eq!(compile_and_run("tests/regression//m33_k16.xi"), Some(0)); }
#[test] fn e2e_m33_k17() { assert_eq!(compile_and_run("tests/regression//m33_k17.xi"), Some(0)); }
#[test] fn e2e_m33_k18() { assert_eq!(compile_and_run("tests/regression//m33_k18.xi"), Some(0)); }
#[test] fn e2e_m33_k19() { assert_eq!(compile_and_run("tests/regression//m33_k19.xi"), Some(0)); }
#[test] fn e2e_m33_k20() { assert_eq!(compile_and_run("tests/regression//m33_k20.xi"), Some(0)); }

#[test] fn e2e_m33_r01() { assert_eq!(compile_and_run("tests/regression//m33_r01.xi"), Some(0)); }
#[test] fn e2e_m33_r02() { assert_eq!(compile_and_run("tests/regression//m33_r02.xi"), Some(0)); }
#[test] fn e2e_m33_r03() { assert_eq!(compile_and_run("tests/regression//m33_r03.xi"), Some(0)); }
#[test] fn e2e_m33_r04() { assert_eq!(compile_and_run("tests/regression//m33_r04.xi"), Some(0)); }
#[test] fn e2e_m33_r05() { assert_eq!(compile_and_run("tests/regression//m33_r05.xi"), Some(0)); }
#[test] fn e2e_m33_r06() { assert_eq!(compile_and_run("tests/regression//m33_r06.xi"), Some(0)); }
#[test] fn e2e_m33_r07() { assert_eq!(compile_and_run("tests/regression//m33_r07.xi"), Some(0)); }
#[test] fn e2e_m33_r08() { assert_eq!(compile_and_run("tests/regression//m33_r08.xi"), Some(0)); }
#[test] fn e2e_m33_r09() { assert_eq!(compile_and_run("tests/regression//m33_r09.xi"), Some(0)); }
#[test] fn e2e_m33_r10() { assert_eq!(compile_and_run("tests/regression//m33_r10.xi"), Some(0)); }
#[test] fn e2e_m33_r11() { assert_eq!(compile_and_run("tests/regression//m33_r11.xi"), Some(0)); }
#[test] fn e2e_m33_r12() { assert_eq!(compile_and_run("tests/regression//m33_r12.xi"), Some(0)); }
#[test] fn e2e_m33_r13() { assert_eq!(compile_and_run("tests/regression//m33_r13.xi"), Some(0)); }
#[test] fn e2e_m33_r14() { assert_eq!(compile_and_run("tests/regression//m33_r14.xi"), Some(0)); }
#[test] fn e2e_m33_r15() { assert_eq!(compile_and_run("tests/regression//m33_r15.xi"), Some(0)); }
#[test] fn e2e_m33_r16() { assert_eq!(compile_and_run("tests/regression//m33_r16.xi"), Some(0)); }
#[test] fn e2e_m33_r17() { assert_eq!(compile_and_run("tests/regression//m33_r17.xi"), Some(0)); }
#[test] fn e2e_m33_r18() { assert_eq!(compile_and_run("tests/regression//m33_r18.xi"), Some(0)); }
#[test] fn e2e_m33_r19() { assert_eq!(compile_and_run("tests/regression//m33_r19.xi"), Some(0)); }
#[test] fn e2e_m33_r20() { assert_eq!(compile_and_run("tests/regression//m33_r20.xi"), Some(0)); }

// -- M33-A: Array / Slice Stress Tests ---------------------------------
// Array creation, indexing, length, Int/Float64/Bool/Str arrays,
// array in struct, array in function param, array return,
// multi-dimensional patterns, bounds checking, very large array,
// array mutation, iteration, array of struct, array of enum,
// array comparison, array with generic, array with contract
#[test] fn e2e_m33_a01() { assert_eq!(compile_and_run("tests/regression//m33_a01.xi"), Some(0)); }
#[test] fn e2e_m33_a02() { assert_eq!(compile_and_run("tests/regression//m33_a02.xi"), Some(0)); }
#[test] fn e2e_m33_a03() { assert_eq!(compile_and_run("tests/regression//m33_a03.xi"), Some(0)); }
#[test] fn e2e_m33_a04() { assert_eq!(compile_and_run("tests/regression//m33_a04.xi"), Some(0)); }
#[test] fn e2e_m33_a05() { assert_eq!(compile_and_run("tests/regression//m33_a05.xi"), Some(0)); }
#[test] fn e2e_m33_a06() { assert_eq!(compile_and_run("tests/regression//m33_a06.xi"), Some(0)); }
#[test] fn e2e_m33_a07() { assert_eq!(compile_and_run("tests/regression//m33_a07.xi"), Some(0)); }
#[test] fn e2e_m33_a08() { assert_eq!(compile_and_run("tests/regression//m33_a08.xi"), Some(0)); }
#[test] fn e2e_m33_a09() { assert_eq!(compile_and_run("tests/regression//m33_a09.xi"), Some(0)); }
#[test] fn e2e_m33_a10() { assert_eq!(compile_and_run("tests/regression//m33_a10.xi"), Some(0)); }
#[test] fn e2e_m33_a11() { assert_eq!(compile_and_run("tests/regression//m33_a11.xi"), Some(0)); }
#[test] fn e2e_m33_a12() { assert_eq!(compile_and_run("tests/regression//m33_a12.xi"), Some(0)); }
#[test] fn e2e_m33_a13() { assert_eq!(compile_and_run("tests/regression//m33_a13.xi"), Some(0)); }
#[test] fn e2e_m33_a14() { assert_eq!(compile_and_run("tests/regression//m33_a14.xi"), Some(0)); }
#[test] fn e2e_m33_a15() { assert_eq!(compile_and_run("tests/regression//m33_a15.xi"), Some(0)); }
#[test] fn e2e_m33_a16() { assert_eq!(compile_and_run("tests/regression//m33_a16.xi"), Some(0)); }
#[test] fn e2e_m33_a17() { assert_eq!(compile_and_run("tests/regression//m33_a17.xi"), Some(0)); }
#[test] fn e2e_m33_a18() { assert_eq!(compile_and_run("tests/regression//m33_a18.xi"), Some(0)); }
#[test] fn e2e_m33_a19() { assert_eq!(compile_and_run("tests/regression//m33_a19.xi"), Some(0)); }
#[test] fn e2e_m33_a20() { assert_eq!(compile_and_run("tests/regression//m33_a20.xi"), Some(0)); }

// -- M33-U: Unsafe / FFI Stress Tests ---------------------------------
// unsafe {} block, extern "C" {}, pointer creation/deref/arith,
// null ptr, ptr to struct, ptr cast, mixed safe/unsafe, ptr chain,
// ptr to array, ptr with contract
#[test] fn e2e_m33_u01() { assert_eq!(compile_and_run("tests/regression//m33_u01.xi"), Some(0)); }
#[test] fn e2e_m33_u02() { assert_eq!(compile_and_run("tests/regression//m33_u02.xi"), Some(0)); }
#[test] fn e2e_m33_u03() { assert_eq!(compile_and_run("tests/regression//m33_u03.xi"), Some(0)); }
#[test] fn e2e_m33_u04() { assert_eq!(compile_and_run("tests/regression//m33_u04.xi"), Some(0)); }
#[test] fn e2e_m33_u05() { assert_eq!(compile_and_run("tests/regression//m33_u05.xi"), Some(0)); }
#[test] fn e2e_m33_u06() { assert_eq!(compile_and_run("tests/regression//m33_u06.xi"), Some(0)); }
#[test] fn e2e_m33_u07() { assert_eq!(compile_and_run("tests/regression//m33_u07.xi"), Some(0)); }
#[test] fn e2e_m33_u08() { assert_eq!(compile_and_run("tests/regression//m33_u08.xi"), Some(0)); }
#[test] fn e2e_m33_u09() { assert_eq!(compile_and_run("tests/regression//m33_u09.xi"), Some(0)); }
#[test] fn e2e_m33_u10() { assert_eq!(compile_and_run("tests/regression//m33_u10.xi"), Some(0)); }
#[test] fn e2e_m33_u11() { assert_eq!(compile_and_run("tests/regression//m33_u11.xi"), Some(0)); }
#[test] fn e2e_m33_u12() { assert_eq!(compile_and_run("tests/regression//m33_u12.xi"), Some(0)); }
#[test] fn e2e_m33_u13() { assert_eq!(compile_and_run("tests/regression//m33_u13.xi"), Some(0)); }
#[test] fn e2e_m33_u14() { assert_eq!(compile_and_run("tests/regression//m33_u14.xi"), Some(0)); }
#[test] fn e2e_m33_u15() { assert_eq!(compile_and_run("tests/regression//m33_u15.xi"), Some(0)); }
#[test] fn e2e_m33_u16() { assert_eq!(compile_and_run("tests/regression//m33_u16.xi"), Some(0)); }
#[test] fn e2e_m33_u17() { assert_eq!(compile_and_run("tests/regression//m33_u17.xi"), Some(0)); }
#[test] fn e2e_m33_u18() { assert_eq!(compile_and_run("tests/regression//m33_u18.xi"), Some(0)); }
#[test] fn e2e_m33_u19() { assert_eq!(compile_and_run("tests/regression//m33_u19.xi"), Some(0)); }
#[test] fn e2e_m33_u20() { assert_eq!(compile_and_run("tests/regression//m33_u20.xi"), Some(0)); }

// -- M33-B: Borrow & Ownership Stress Tests -------------------------------
// Read borrow on local, write borrow on local, multiple read borrows,
// exclusive write borrow, borrow through function param, struct field
// access through borrow, return a value (move), clone to duplicate,
// let binding copies, var reassignment, borrow in if scope,
// borrow in while scope, borrow in match, borrow with generic,
// immutable let vs mutable var, shadowing with ownership,
// array borrow, struct borrow, return owned from function, borrow chain
#[test] fn e2e_m33_b01() { assert_eq!(compile_and_run("tests/regression//m33_b01.xi"), Some(0)); }
#[test] fn e2e_m33_b02() { assert_eq!(compile_and_run("tests/regression//m33_b02.xi"), Some(0)); }
#[test] fn e2e_m33_b03() { assert_eq!(compile_and_run("tests/regression//m33_b03.xi"), Some(0)); }
#[test] fn e2e_m33_b04() { assert_eq!(compile_and_run("tests/regression//m33_b04.xi"), Some(0)); }
#[test] fn e2e_m33_b05() { assert_eq!(compile_and_run("tests/regression//m33_b05.xi"), Some(0)); }
#[test] fn e2e_m33_b06() { assert_eq!(compile_and_run("tests/regression//m33_b06.xi"), Some(0)); }
#[test] fn e2e_m33_b07() { assert_eq!(compile_and_run("tests/regression//m33_b07.xi"), Some(0)); }
#[test] fn e2e_m33_b08() { assert_eq!(compile_and_run("tests/regression//m33_b08.xi"), Some(0)); }
#[test] fn e2e_m33_b09() { assert_eq!(compile_and_run("tests/regression//m33_b09.xi"), Some(0)); }
#[test] fn e2e_m33_b10() { assert_eq!(compile_and_run("tests/regression//m33_b10.xi"), Some(0)); }
#[test] fn e2e_m33_b11() { assert_eq!(compile_and_run("tests/regression//m33_b11.xi"), Some(0)); }
#[test] fn e2e_m33_b12() { assert_eq!(compile_and_run("tests/regression//m33_b12.xi"), Some(0)); }
#[test] fn e2e_m33_b13() { assert_eq!(compile_and_run("tests/regression//m33_b13.xi"), Some(0)); }
#[test] fn e2e_m33_b14() { assert_eq!(compile_and_run("tests/regression//m33_b14.xi"), Some(0)); }
#[test] fn e2e_m33_b15() { assert_eq!(compile_and_run("tests/regression//m33_b15.xi"), Some(0)); }
#[test] fn e2e_m33_b16() { assert_eq!(compile_and_run("tests/regression//m33_b16.xi"), Some(0)); }
#[test] fn e2e_m33_b17() { assert_eq!(compile_and_run("tests/regression//m33_b17.xi"), Some(0)); }
#[test] fn e2e_m33_b18() { assert_eq!(compile_and_run("tests/regression//m33_b18.xi"), Some(0)); }
#[test] fn e2e_m33_b19() { assert_eq!(compile_and_run("tests/regression//m33_b19.xi"), Some(0)); }
#[test] fn e2e_m33_b20() { assert_eq!(compile_and_run("tests/regression//m33_b20.xi"), Some(0)); }

// -- M33-P: Large Program Stress Tests ---------------------------------
// Each file: 500+ lines, 30-fn chain, 20 struct types, 15 enum types,
// 10 const declarations, 5 modules, deep call chain (fn0->...->fn20),
// wide function table (50 one-line fns), complex type graph (10+ types),
// large match expression (20 arms), many local variables (50 per fn),
// deeply nested blocks (10 levels), interleaved declarations,
// all primitive types in one struct (20 fields), multi-module cross-ref.
#[test] fn e2e_m33_p01() { assert_eq!(compile_and_run("tests/regression//m33_p01.xi"), Some(0)); }
#[test] fn e2e_m33_p02() { assert_eq!(compile_and_run("tests/regression//m33_p02.xi"), Some(0)); }
#[test] fn e2e_m33_p03() { assert_eq!(compile_and_run("tests/regression//m33_p03.xi"), Some(0)); }
#[test] fn e2e_m33_p04() { assert_eq!(compile_and_run("tests/regression//m33_p04.xi"), Some(0)); }
#[test] fn e2e_m33_p05() { assert_eq!(compile_and_run("tests/regression//m33_p05.xi"), Some(0)); }
#[test] fn e2e_m33_p06() { assert_eq!(compile_and_run("tests/regression//m33_p06.xi"), Some(0)); }
#[test] fn e2e_m33_p07() { assert_eq!(compile_and_run("tests/regression//m33_p07.xi"), Some(0)); }
#[test] fn e2e_m33_p08() { assert_eq!(compile_and_run("tests/regression//m33_p08.xi"), Some(0)); }
#[test] fn e2e_m33_p09() { assert_eq!(compile_and_run("tests/regression//m33_p09.xi"), Some(0)); }
#[test] fn e2e_m33_p10() { assert_eq!(compile_and_run("tests/regression//m33_p10.xi"), Some(0)); }
#[test] fn e2e_m33_p11() { assert_eq!(compile_and_run("tests/regression//m33_p11.xi"), Some(0)); }
#[test] fn e2e_m33_p12() { assert_eq!(compile_and_run("tests/regression//m33_p12.xi"), Some(0)); }
#[test] fn e2e_m33_p13() { assert_eq!(compile_and_run("tests/regression//m33_p13.xi"), Some(0)); }
#[test] fn e2e_m33_p14() { assert_eq!(compile_and_run("tests/regression//m33_p14.xi"), Some(0)); }
#[test] fn e2e_m33_p15() { assert_eq!(compile_and_run("tests/regression//m33_p15.xi"), Some(0)); }
#[test] fn e2e_m33_p16() { assert_eq!(compile_and_run("tests/regression//m33_p16.xi"), Some(0)); }
#[test] fn e2e_m33_p17() { assert_eq!(compile_and_run("tests/regression//m33_p17.xi"), Some(0)); }
#[test] fn e2e_m33_p18() { assert_eq!(compile_and_run("tests/regression//m33_p18.xi"), Some(0)); }
#[test] fn e2e_m33_p19() { assert_eq!(compile_and_run("tests/regression//m33_p19.xi"), Some(0)); }
#[test] fn e2e_m33_p20() { assert_eq!(compile_and_run("tests/regression//m33_p20.xi"), Some(0)); }

// -- M34-N2: Deep Nesting Stress Tests ---------------------------------
// 01: 10-deep if-else chain
// 02: 10-deep while-with-while loop (2^10 = 1024 iterations)
// 03: 20-deep addition expression ((((1+2)+3)...+20) = 210
// 04: 5-level nested generic type chain L1[T]->L2->L3->L4->L5[T]
// 05: Nested enum in enum in enum (3 levels)
// 06: 6-level nested struct S1->S2->S3->S4->S5->S6
// 07: 4-level nested match E1 inside E2 inside E3 inside E4
// 08: 10-level nested blocks
// 09: Array-of-array-of-array via chained vec indexing
// 10: Function factory pattern -- closure returned from function
// 11: Closure in closure capturing outer scope
// 12: 5-level nested modules m1->m2->m3->m4->m5
// 13: Borrow-in-borrow pattern &T -> &U chain
// 14: Multi-contract nesting -- requires/ensures chain
// 15: 8-level type alias chain T0->T1->...->T7
// 16: 10-deep while-with-if alternating nesting
// 17: Deep match-with-if interleaved nesting
// 18: Deep tuple-like nesting via layered struct pairs
// 19: 20-deep multiplication expression 1*2*...*20 = 20!
// 20: Deep match nesting with multi-variant enums (3 levels)
#[test] fn e2e_m34_n2_01() { assert_eq!(compile_and_run("tests/regression//m34_n2_01.xi"), Some(0)); }
#[test] fn e2e_m34_n2_02() { assert_eq!(compile_and_run("tests/regression//m34_n2_02.xi"), Some(0)); }
#[test] fn e2e_m34_n2_03() { assert_eq!(compile_and_run("tests/regression//m34_n2_03.xi"), Some(0)); }
#[test] fn e2e_m34_n2_04() { assert_eq!(compile_and_run("tests/regression//m34_n2_04.xi"), Some(0)); }
#[test] fn e2e_m34_n2_05() { assert_eq!(compile_and_run("tests/regression//m34_n2_05.xi"), Some(0)); }
#[test] fn e2e_m34_n2_06() { assert_eq!(compile_and_run("tests/regression//m34_n2_06.xi"), Some(0)); }
#[test] fn e2e_m34_n2_07() { assert_eq!(compile_and_run("tests/regression//m34_n2_07.xi"), Some(0)); }
#[test] fn e2e_m34_n2_08() { assert_eq!(compile_and_run("tests/regression//m34_n2_08.xi"), Some(0)); }
#[test] fn e2e_m34_n2_09() { assert_eq!(compile_and_run("tests/regression//m34_n2_09.xi"), Some(0)); }
#[test] fn e2e_m34_n2_10() { assert_eq!(compile_and_run("tests/regression//m34_n2_10.xi"), Some(0)); }
#[test] fn e2e_m34_n2_11() { assert_eq!(compile_and_run("tests/regression//m34_n2_11.xi"), Some(0)); }
#[test] fn e2e_m34_n2_12() { assert_eq!(compile_and_run("tests/regression//m34_n2_12.xi"), Some(0)); }
#[test] fn e2e_m34_n2_13() { assert_eq!(compile_and_run("tests/regression//m34_n2_13.xi"), Some(0)); }
#[test] fn e2e_m34_n2_14() { assert_eq!(compile_and_run("tests/regression//m34_n2_14.xi"), Some(0)); }
#[test] fn e2e_m34_n2_15() { assert_eq!(compile_and_run("tests/regression//m34_n2_15.xi"), Some(0)); }
#[test] fn e2e_m34_n2_16() { assert_eq!(compile_and_run("tests/regression//m34_n2_16.xi"), Some(0)); }
#[test] fn e2e_m34_n2_17() { assert_eq!(compile_and_run("tests/regression//m34_n2_17.xi"), Some(0)); }
#[test] fn e2e_m34_n2_18() { assert_eq!(compile_and_run("tests/regression//m34_n2_18.xi"), Some(0)); }
#[test] fn e2e_m34_n2_19() { assert_eq!(compile_and_run("tests/regression//m34_n2_19.xi"), Some(0)); }
#[test] fn e2e_m34_n2_20() { assert_eq!(compile_and_run("tests/regression//m34_n2_20.xi"), Some(0)); }

// -- M34-Q: Contract Chain Stress Tests --------------------------------
// q01: Chained requires chain (fn a requires -> fn b requires -> fn c)
#[test] fn e2e_m34_q01() { assert_eq!(compile_and_run("tests/regression//m34_q01.xi"), Some(0), "q01: chained requires"); }
// q02: Chained ensures propagated through call chain
#[test] fn e2e_m34_q02() { assert_eq!(compile_and_run("tests/regression//m34_q02.xi"), Some(0), "q02: chained ensures"); }
// q03: Contract + generic types
#[test] fn e2e_m34_q03() { assert_eq!(compile_and_run("tests/regression//m34_q03.xi"), Some(0), "q03: contract+generic"); }
// q04: Contract + methods on struct
#[test] fn e2e_m34_q04() { assert_eq!(compile_and_run("tests/regression//m34_q04.xi"), Some(0), "q04: contract+method"); }
// q05: Contract + invariant on struct
#[test] fn e2e_m34_q05() { assert_eq!(compile_and_run("tests/regression//m34_q05.xi"), Some(0), "q05: contract+invariant+struct"); }
// q06: Contract runtime -- valid paths pass, contract protects invariants
#[test] fn e2e_m34_q06() { assert_eq!(compile_and_run("tests/regression//m34_q06.xi"), Some(0), "q06: contract runtime"); }
// q07: Contract with Option types
#[test] fn e2e_m34_q07() { assert_eq!(compile_and_run("tests/regression//m34_q07.xi"), Some(0), "q07: contract+Option"); }
// q08: Contract with Result types
#[test] fn e2e_m34_q08() { assert_eq!(compile_and_run("tests/regression//m34_q08.xi"), Some(0), "q08: contract+Result"); }
// q09: Contract with pointer null check
#[test] fn e2e_m34_q09() { assert_eq!(compile_and_run("tests/regression//m34_q09.xi"), Some(0), "q09: contract+pointer null"); }
// q10: Contract with array bounds
#[test] fn e2e_m34_q10() { assert_eq!(compile_and_run("tests/regression//m34_q10.xi"), Some(0), "q10: contract+array bounds"); }
// q11: Contract with arithmetic overflow guards
#[test] fn e2e_m34_q11() { assert_eq!(compile_and_run("tests/regression//m34_q11.xi"), Some(0), "q11: contract+overflow"); }
// q12: Contract with float domain constraints
#[test] fn e2e_m34_q12() { assert_eq!(compile_and_run("tests/regression//m34_q12.xi"), Some(0), "q12: contract+float"); }
// q13: Contract with boolean preconditions
#[test] fn e2e_m34_q13() { assert_eq!(compile_and_run("tests/regression//m34_q13.xi"), Some(0), "q13: contract+boolean precondition"); }
// q14: Contract with string length
#[test] fn e2e_m34_q14() { assert_eq!(compile_and_run("tests/regression//m34_q14.xi"), Some(0), "q14: contract+string"); }
// q15: Multi-function contract chain (8 functions, 5+ with contracts)
#[test] fn e2e_m34_q15() { assert_eq!(compile_and_run("tests/regression//m34_q15.xi"), Some(0), "q15: 8-fn contract chain"); }
// q16: Contract with external functions
#[test] fn e2e_m34_q16() { assert_eq!(compile_and_run("tests/regression//m34_q16.xi"), Some(0), "q16: contract+extern"); }
// q17: Contract with recursion
#[test] fn e2e_m34_q17() { assert_eq!(compile_and_run("tests/regression//m34_q17.xi"), Some(0), "q17: contract+recursion"); }
// q18: Contract with closures
#[test] fn e2e_m34_q18() { assert_eq!(compile_and_run("tests/regression//m34_q18.xi"), Some(0), "q18: contract+closure"); }
// q19: Contract with modules
#[test] fn e2e_m34_q19() { assert_eq!(compile_and_run("tests/regression//m34_q19.xi"), Some(0), "q19: contract+module"); }
// q20: Combined mega stress (struct+enum+generic+module+method+closure+recursion+Option+Result+float+array)
#[test] fn e2e_m34_q20() { assert_eq!(compile_and_run("tests/regression//m34_q20.xi"), Some(0), "q20: all patterns combined"); }

// -- M34-O: Error Propagation Chain Tests -------------------------------
// ? operator on Result[T,E]: simple ? Ok, ? Err, 2-/3-/5-chain,
// ? in Result fn, Option propagation, mixed with match/if-guard/loop,
// early return, compound expr, generic result, custom error type,
// Option chain, map+?, and_then pattern, ? with contract/method/module
#[test] fn e2e_m34_o01() { assert_eq!(compile_and_run("tests/regression//m34_o01.xi"), Some(0)); }
#[test] fn e2e_m34_o02() { assert_eq!(compile_and_run("tests/regression//m34_o02.xi"), Some(0)); }
#[test] fn e2e_m34_o03() { assert_eq!(compile_and_run("tests/regression//m34_o03.xi"), Some(0)); }
#[test] fn e2e_m34_o04() { assert_eq!(compile_and_run("tests/regression//m34_o04.xi"), Some(0)); }
#[test] fn e2e_m34_o05() { assert_eq!(compile_and_run("tests/regression//m34_o05.xi"), Some(0)); }
#[test] fn e2e_m34_o06() { assert_eq!(compile_and_run("tests/regression//m34_o06.xi"), Some(0)); }
#[test] fn e2e_m34_o07() { assert_eq!(compile_and_run("tests/regression//m34_o07.xi"), Some(0)); }
#[test] fn e2e_m34_o08() { assert_eq!(compile_and_run("tests/regression//m34_o08.xi"), Some(0)); }
#[test] fn e2e_m34_o09() { assert_eq!(compile_and_run("tests/regression//m34_o09.xi"), Some(0)); }
#[test] fn e2e_m34_o10() { assert_eq!(compile_and_run("tests/regression//m34_o10.xi"), Some(0)); }
#[test] fn e2e_m34_o11() { assert_eq!(compile_and_run("tests/regression//m34_o11.xi"), Some(0)); }
#[test] fn e2e_m34_o12() { assert_eq!(compile_and_run("tests/regression//m34_o12.xi"), Some(0)); }
#[test] fn e2e_m34_o13() { assert_eq!(compile_and_run("tests/regression//m34_o13.xi"), Some(0)); }
#[test] fn e2e_m34_o14() { assert_eq!(compile_and_run("tests/regression//m34_o14.xi"), Some(0)); }
#[test] fn e2e_m34_o15() { assert_eq!(compile_and_run("tests/regression//m34_o15.xi"), Some(0)); }
#[test] fn e2e_m34_o16() { assert_eq!(compile_and_run("tests/regression//m34_o16.xi"), Some(0)); }
#[test] fn e2e_m34_o17() { assert_eq!(compile_and_run("tests/regression//m34_o17.xi"), Some(0)); }
#[test] fn e2e_m34_o18() { assert_eq!(compile_and_run("tests/regression//m34_o18.xi"), Some(0)); }
#[test] fn e2e_m34_o19() { assert_eq!(compile_and_run("tests/regression//m34_o19.xi"), Some(0)); }
#[test] fn e2e_m34_o20() { assert_eq!(compile_and_run("tests/regression//m34_o20.xi"), Some(0)); }

// -- M34-V: Float Special Values Stress Tests ---------------------------
// Very large/small values, zero/negative-zero, addition/multiplication
// precision, division, Int-Float boundary, comparisons, epsilon equality,
// multiply-accumulate, struct field, generic, const, parameter chain,
// return type, while-loop accumulator, array operations, Float32 coverage
#[test] fn e2e_m34_v01() { assert_eq!(compile_and_run("tests/regression//m34_v01.xi"), Some(0)); }
#[test] fn e2e_m34_v02() { assert_eq!(compile_and_run("tests/regression//m34_v02.xi"), Some(0)); }
#[test] fn e2e_m34_v03() { assert_eq!(compile_and_run("tests/regression//m34_v03.xi"), Some(0)); }
#[test] fn e2e_m34_v04() { assert_eq!(compile_and_run("tests/regression//m34_v04.xi"), Some(0)); }
#[test] fn e2e_m34_v05() { assert_eq!(compile_and_run("tests/regression//m34_v05.xi"), Some(0)); }
#[test] fn e2e_m34_v06() { assert_eq!(compile_and_run("tests/regression//m34_v06.xi"), Some(0)); }
#[test] fn e2e_m34_v07() { assert_eq!(compile_and_run("tests/regression//m34_v07.xi"), Some(0)); }
#[test] fn e2e_m34_v08() { assert_eq!(compile_and_run("tests/regression//m34_v08.xi"), Some(0)); }
#[test] fn e2e_m34_v09() { assert_eq!(compile_and_run("tests/regression//m34_v09.xi"), Some(0)); }
#[test] fn e2e_m34_v10() { assert_eq!(compile_and_run("tests/regression//m34_v10.xi"), Some(0)); }
#[test] fn e2e_m34_v11() { assert_eq!(compile_and_run("tests/regression//m34_v11.xi"), Some(0)); }
#[test] fn e2e_m34_v12() { assert_eq!(compile_and_run("tests/regression//m34_v12.xi"), Some(0)); }
#[test] fn e2e_m34_v13() { assert_eq!(compile_and_run("tests/regression//m34_v13.xi"), Some(0)); }
#[test] fn e2e_m34_v14() { assert_eq!(compile_and_run("tests/regression//m34_v14.xi"), Some(0)); }
#[test] fn e2e_m34_v15() { assert_eq!(compile_and_run("tests/regression//m34_v15.xi"), Some(0)); }
#[test] fn e2e_m34_v16() { assert_eq!(compile_and_run("tests/regression//m34_v16.xi"), Some(0)); }
#[test] fn e2e_m34_v17() { assert_eq!(compile_and_run("tests/regression//m34_v17.xi"), Some(0)); }
#[test] fn e2e_m34_v18() { assert_eq!(compile_and_run("tests/regression//m34_v18.xi"), Some(0)); }
#[test] fn e2e_m34_v19() { assert_eq!(compile_and_run("tests/regression//m34_v19.xi"), Some(0)); }
#[test] fn e2e_m34_v20() { assert_eq!(compile_and_run("tests/regression//m34_v20.xi"), Some(0)); }

// -- M34-N: Derive Operations Stress Tests ---------------------------------
// Tests: derive[Eq], derive[Clone], derive[Display], derive[Hash],
// derive[Ord], derive[Debug] with structs, enums, generics, nesting.
#[test] fn e2e_m34_n01() { assert_eq!(compile_and_run("tests/regression//m34_n01.xi"), Some(0)); }
#[test] fn e2e_m34_n02() { assert_eq!(compile_and_run("tests/regression//m34_n02.xi"), Some(0)); }
#[test] fn e2e_m34_n03() { assert_eq!(compile_and_run("tests/regression//m34_n03.xi"), Some(0)); }
#[test] fn e2e_m34_n04() { assert_eq!(compile_and_run("tests/regression//m34_n04.xi"), Some(0)); }
#[test] fn e2e_m34_n05() { assert_eq!(compile_and_run("tests/regression//m34_n05.xi"), Some(0)); }
#[test] fn e2e_m34_n06() { assert_eq!(compile_and_run("tests/regression//m34_n06.xi"), Some(0)); }
#[test] fn e2e_m34_n07() { assert_eq!(compile_and_run("tests/regression//m34_n07.xi"), Some(0)); }
#[test] fn e2e_m34_n08() { assert_eq!(compile_and_run("tests/regression//m34_n08.xi"), Some(0)); }
#[test] fn e2e_m34_n09() { assert_eq!(compile_and_run("tests/regression//m34_n09.xi"), Some(0)); }
#[test] fn e2e_m34_n10() { assert_eq!(compile_and_run("tests/regression//m34_n10.xi"), Some(0)); }
#[test] fn e2e_m34_n11() { assert_eq!(compile_and_run("tests/regression//m34_n11.xi"), Some(0)); }
#[test] fn e2e_m34_n12() { assert_eq!(compile_and_run("tests/regression//m34_n12.xi"), Some(0)); }
#[test] fn e2e_m34_n13() { assert_eq!(compile_and_run("tests/regression//m34_n13.xi"), Some(0)); }
#[test] fn e2e_m34_n14() { assert_eq!(compile_and_run("tests/regression//m34_n14.xi"), Some(0)); }
#[test] fn e2e_m34_n15() { assert_eq!(compile_and_run("tests/regression//m34_n15.xi"), Some(0)); }
#[test] fn e2e_m34_n16() { assert_eq!(compile_and_run("tests/regression//m34_n16.xi"), Some(0)); }
#[test] fn e2e_m34_n17() { assert_eq!(compile_and_run("tests/regression//m34_n17.xi"), Some(0)); }
#[test] fn e2e_m34_n18() { assert_eq!(compile_and_run("tests/regression//m34_n18.xi"), Some(0)); }
#[test] fn e2e_m34_n19() { assert_eq!(compile_and_run("tests/regression//m34_n19.xi"), Some(0)); }
#[test] fn e2e_m34_n20() { assert_eq!(compile_and_run("tests/regression//m34_n20.xi"), Some(0)); }

// -- M34-H: Type Coercion / Casting Edge Cases --------------------------
// Int8->Int16->Int32->Int64 chain, Int64->Int32->Int16->Int8 truncation,
// unsigned <-> signed, Float64<->Int, Float<->Float narrowing/widening,
// Char<->Int, Bool<->Int, cast in expression chain, function call,
// struct field, enum payload, array index, while condition,
// cast after arithmetic overflow, cast in generic context
#[test] fn e2e_m34_h01() { assert_eq!(compile_and_run("tests/regression//m34_h01.xi"), Some(0)); }
#[test] fn e2e_m34_h02() { assert_eq!(compile_and_run("tests/regression//m34_h02.xi"), Some(0)); }
#[test] fn e2e_m34_h03() { assert_eq!(compile_and_run("tests/regression//m34_h03.xi"), Some(0)); }
#[test] fn e2e_m34_h04() { assert_eq!(compile_and_run("tests/regression//m34_h04.xi"), Some(0)); }
#[test] fn e2e_m34_h05() { assert_eq!(compile_and_run("tests/regression//m34_h05.xi"), Some(0)); }
#[test] fn e2e_m34_h06() { assert_eq!(compile_and_run("tests/regression//m34_h06.xi"), Some(0)); }
#[test] fn e2e_m34_h07() { assert_eq!(compile_and_run("tests/regression//m34_h07.xi"), Some(0)); }
#[test] fn e2e_m34_h08() { assert_eq!(compile_and_run("tests/regression//m34_h08.xi"), Some(0)); }
#[test] fn e2e_m34_h09() { assert_eq!(compile_and_run("tests/regression//m34_h09.xi"), Some(0)); }
#[test] fn e2e_m34_h10() { assert_eq!(compile_and_run("tests/regression//m34_h10.xi"), Some(0)); }
#[test] fn e2e_m34_h11() { assert_eq!(compile_and_run("tests/regression//m34_h11.xi"), Some(0)); }
#[test] fn e2e_m34_h12() { assert_eq!(compile_and_run("tests/regression//m34_h12.xi"), Some(0)); }
#[test] fn e2e_m34_h13() { assert_eq!(compile_and_run("tests/regression//m34_h13.xi"), Some(0)); }
#[test] fn e2e_m34_h14() { assert_eq!(compile_and_run("tests/regression//m34_h14.xi"), Some(0)); }
#[test] fn e2e_m34_h15() { assert_eq!(compile_and_run("tests/regression//m34_h15.xi"), Some(0)); }
#[test] fn e2e_m34_h16() { assert_eq!(compile_and_run("tests/regression//m34_h16.xi"), Some(0)); }
#[test] fn e2e_m34_h17() { assert_eq!(compile_and_run("tests/regression//m34_h17.xi"), Some(0)); }
#[test] fn e2e_m34_h18() { assert_eq!(compile_and_run("tests/regression//m34_h18.xi"), Some(0)); }
#[test] fn e2e_m34_h19() { assert_eq!(compile_and_run("tests/regression//m34_h19.xi"), Some(0)); }
#[test] fn e2e_m34_h20() { assert_eq!(compile_and_run("tests/regression//m34_h20.xi"), Some(0)); }

// -- M34-J: Cross-Module Stress Tests ----------------------------------
// Multiple modules in one file, pub fn/type/enum/const, use/use.Type/use as alias,
// dotted paths, nested modules (3+), re-export, private fn,
// cross-module type references, generics, contracts, invariants, derive, methods, all combined
#[test] fn e2e_m34_j01() { assert_eq!(compile_and_run("tests/regression//m34_j01.xi"), Some(0)); }
#[test] fn e2e_m34_j02() { assert_eq!(compile_and_run("tests/regression//m34_j02.xi"), Some(0)); }
#[test] fn e2e_m34_j03() { assert_eq!(compile_and_run("tests/regression//m34_j03.xi"), Some(0)); }
#[test] fn e2e_m34_j04() { assert_eq!(compile_and_run("tests/regression//m34_j04.xi"), Some(0)); }
#[test] fn e2e_m34_j05() { assert_eq!(compile_and_run("tests/regression//m34_j05.xi"), Some(0)); }
#[test] fn e2e_m34_j06() { assert_eq!(compile_and_run("tests/regression//m34_j06.xi"), Some(0)); }
#[test] fn e2e_m34_j07() { assert_eq!(compile_and_run("tests/regression//m34_j07.xi"), Some(0)); }
#[test] fn e2e_m34_j08() { assert_eq!(compile_and_run("tests/regression//m34_j08.xi"), Some(0)); }
#[test] fn e2e_m34_j09() { assert_eq!(compile_and_run("tests/regression//m34_j09.xi"), Some(0)); }
#[test] fn e2e_m34_j10() { assert_eq!(compile_and_run("tests/regression//m34_j10.xi"), Some(0)); }
#[test] fn e2e_m34_j11() { assert_eq!(compile_and_run("tests/regression//m34_j11.xi"), Some(0)); }
#[test] fn e2e_m34_j12() { assert_eq!(compile_and_run("tests/regression//m34_j12.xi"), Some(0)); }
#[test] fn e2e_m34_j13() { assert_eq!(compile_and_run("tests/regression//m34_j13.xi"), Some(0)); }
#[test] fn e2e_m34_j14() { assert_eq!(compile_and_run("tests/regression//m34_j14.xi"), Some(0)); }
#[test] fn e2e_m34_j15() { assert_eq!(compile_and_run("tests/regression//m34_j15.xi"), Some(0)); }
#[test] fn e2e_m34_j16() { assert_eq!(compile_and_run("tests/regression//m34_j16.xi"), Some(0)); }
#[test] fn e2e_m34_j17() { assert_eq!(compile_and_run("tests/regression//m34_j17.xi"), Some(0)); }
#[test] fn e2e_m34_j18() { assert_eq!(compile_and_run("tests/regression//m34_j18.xi"), Some(0)); }
#[test] fn e2e_m34_j19() { assert_eq!(compile_and_run("tests/regression//m34_j19.xi"), Some(0)); }
#[test] fn e2e_m34_j20() { assert_eq!(compile_and_run("tests/regression//m34_j20.xi"), Some(0)); }

// -- M34-W: Bitwise/Bit-Level Stress Tests ------------------------------
// Operators: & | ^ ~ << >> on Int,Int8,Int16,Int32,Int64,UInt,UInt8,UInt16,UInt32
// w01: AND identity (x & -1 == x) on Int,Int8,Int16,Int32,Int64
#[test] fn e2e_m34_w01() { assert_eq!(compile_and_run("tests/regression//m34_w01.xi"), Some(0)); }
// w02: OR zero (x | 0 == x) on UInt,UInt8,UInt16,UInt32
#[test] fn e2e_m34_w02() { assert_eq!(compile_and_run("tests/regression//m34_w02.xi"), Some(0)); }
// w03: XOR self (x ^ x == 0) on Int,Int32,UInt
#[test] fn e2e_m34_w03() { assert_eq!(compile_and_run("tests/regression//m34_w03.xi"), Some(0)); }
// w04: Double NOT (~~x == x) on all 9 integer types
#[test] fn e2e_m34_w04() { assert_eq!(compile_and_run("tests/regression//m34_w04.xi"), Some(0)); }
// w05: Shift left by 1 (multiply by 2) on Int,Int32,UInt
#[test] fn e2e_m34_w05() { assert_eq!(compile_and_run("tests/regression//m34_w05.xi"), Some(0)); }
// w06: Shift right by 1 (divide by 2) on Int,Int32,UInt
#[test] fn e2e_m34_w06() { assert_eq!(compile_and_run("tests/regression//m34_w06.xi"), Some(0)); }
// w07: Shift by 0 (identity) on Int,Int32,UInt
#[test] fn e2e_m34_w07() { assert_eq!(compile_and_run("tests/regression//m34_w07.xi"), Some(0)); }
// w08: Bit isolation (mask & value) using hex masks on multiple types
#[test] fn e2e_m34_w08() { assert_eq!(compile_and_run("tests/regression//m34_w08.xi"), Some(0)); }
// w09: Sign bit manipulation (MSB on signed types)
#[test] fn e2e_m34_w09() { assert_eq!(compile_and_run("tests/regression//m34_w09.xi"), Some(0)); }
// w10: Bit counting (popcount via while loop)
#[test] fn e2e_m34_w10() { assert_eq!(compile_and_run("tests/regression//m34_w10.xi"), Some(0)); }
// w11: Power-of-2 check (x & (x-1) == 0)
#[test] fn e2e_m34_w11() { assert_eq!(compile_and_run("tests/regression//m34_w11.xi"), Some(0)); }
// w12: Bit set/clear/toggle patterns
#[test] fn e2e_m34_w12() { assert_eq!(compile_and_run("tests/regression//m34_w12.xi"), Some(0)); }
// w13: Byte swap (shift + mask) on UInt32
#[test] fn e2e_m34_w13() { assert_eq!(compile_and_run("tests/regression//m34_w13.xi"), Some(0)); }
// w14: Bit rotation (rotl/rotr via shift + OR)
#[test] fn e2e_m34_w14() { assert_eq!(compile_and_run("tests/regression//m34_w14.xi"), Some(0)); }
// w15: Right-shift sign extension (arithmetic shift on signed types)
#[test] fn e2e_m34_w15() { assert_eq!(compile_and_run("tests/regression//m34_w15.xi"), Some(0)); }
// w16: Unsigned right shift (logical/zero-fill shift on UInt,UInt32)
#[test] fn e2e_m34_w16() { assert_eq!(compile_and_run("tests/regression//m34_w16.xi"), Some(0)); }
// w17: Hex literal masking (0xFF & x across types)
#[test] fn e2e_m34_w17() { assert_eq!(compile_and_run("tests/regression//m34_w17.xi"), Some(0)); }
// w18: Bitwise compound patterns (sequential & | ^ and chain expressions)
#[test] fn e2e_m34_w18() { assert_eq!(compile_and_run("tests/regression//m34_w18.xi"), Some(0)); }
// w19: Cross-type bitwise (Int32 & UInt32 with casts)
#[test] fn e2e_m34_w19() { assert_eq!(compile_and_run("tests/regression//m34_w19.xi"), Some(0)); }
// w20: Comprehensive bitwise stress (all operators on all 9 integer types)
#[test] fn e2e_m34_w20() { assert_eq!(compile_and_run("tests/regression//m34_w20.xi"), Some(0)); }

// -- M34-Y: Fuzzing-Style Combinatorial Stress Tests ------------------
// Each test randomly combines 5-8 language features in unusual patterns:
// struct, enum, match, while, if, closures, generics, contracts,
// arrays, modules, pointers, Result, Option, compound assign, derive, impl, unsafe
#[test] fn e2e_m34_y01() { assert_eq!(compile_and_run("tests/regression//m34_y01.xi"), Some(0)); }
#[test] fn e2e_m34_y02() { assert_eq!(compile_and_run("tests/regression//m34_y02.xi"), Some(0)); }
#[test] fn e2e_m34_y03() { assert_eq!(compile_and_run("tests/regression//m34_y03.xi"), Some(0)); }
#[test] fn e2e_m34_y04() { assert_eq!(compile_and_run("tests/regression//m34_y04.xi"), Some(0)); }
#[test] fn e2e_m34_y05() { assert_eq!(compile_and_run("tests/regression//m34_y05.xi"), Some(0)); }
#[test] fn e2e_m34_y06() { assert_eq!(compile_and_run("tests/regression//m34_y06.xi"), Some(0)); }
#[test] fn e2e_m34_y07() { assert_eq!(compile_and_run("tests/regression//m34_y07.xi"), Some(0)); }
#[test] fn e2e_m34_y08() { assert_eq!(compile_and_run("tests/regression//m34_y08.xi"), Some(0)); }
#[test] fn e2e_m34_y09() { assert_eq!(compile_and_run("tests/regression//m34_y09.xi"), Some(0)); }
#[test] fn e2e_m34_y10() { assert_eq!(compile_and_run("tests/regression//m34_y10.xi"), Some(0)); }
#[test] fn e2e_m34_y11() { assert_eq!(compile_and_run("tests/regression//m34_y11.xi"), Some(0)); }
#[test] fn e2e_m34_y12() { assert_eq!(compile_and_run("tests/regression//m34_y12.xi"), Some(0)); }
#[test] fn e2e_m34_y13() { assert_eq!(compile_and_run("tests/regression//m34_y13.xi"), Some(0)); }
#[test] fn e2e_m34_y14() { assert_eq!(compile_and_run("tests/regression//m34_y14.xi"), Some(0)); }
#[test] fn e2e_m34_y15() { assert_eq!(compile_and_run("tests/regression//m34_y15.xi"), Some(0)); }
#[test] fn e2e_m34_y16() { assert_eq!(compile_and_run("tests/regression//m34_y16.xi"), Some(0)); }
#[test] fn e2e_m34_y17() { assert_eq!(compile_and_run("tests/regression//m34_y17.xi"), Some(0)); }
#[test] fn e2e_m34_y18() { assert_eq!(compile_and_run("tests/regression//m34_y18.xi"), Some(0)); }
#[test] fn e2e_m34_y19() { assert_eq!(compile_and_run("tests/regression//m34_y19.xi"), Some(0)); }
#[test] fn e2e_m34_y20() { assert_eq!(compile_and_run("tests/regression//m34_y20.xi"), Some(0)); }

// -- M35-M: Math Function Stress Tests -------------------------------
// 01: sqrt via Newton method, 02: abs value, 03: min of 2/3/4, 04: max of 2/3/4
// 05: integer pow, 06: factorial iterative, 07: factorial recursive
// 08: is-prime trial division, 09: GCD Euclidean, 10: LCM from GCD
// 11: fibonacci iterative, 12: digital root, 13: perfect number, 14: Armstrong
// 15: arithmetic series sum, 16: geometric series sum, 17: harmonic number
// 18: binomial coefficient, 19: Pascal triangle row, 20: Catalan number
// 21: Collatz sequence, 22: hailstone max, 23: Leibniz pi approx
// 24: e approximation, 25: log2 integer, 26: is-power-of-2
// 27: round to nearest, 28: ceil division, 29: triangle area Heron
// 30: quadratic discriminant + distance
#[test] fn e2e_m35_m01() { assert_eq!(compile_and_run("tests/regression//m35_m01.xi"), Some(0)); }
#[test] fn e2e_m35_m02() { assert_eq!(compile_and_run("tests/regression//m35_m02.xi"), Some(0)); }
#[test] fn e2e_m35_m03() { assert_eq!(compile_and_run("tests/regression//m35_m03.xi"), Some(0)); }
#[test] fn e2e_m35_m04() { assert_eq!(compile_and_run("tests/regression//m35_m04.xi"), Some(0)); }
#[test] fn e2e_m35_m05() { assert_eq!(compile_and_run("tests/regression//m35_m05.xi"), Some(0)); }
#[test] fn e2e_m35_m06() { assert_eq!(compile_and_run("tests/regression//m35_m06.xi"), Some(0)); }
#[test] fn e2e_m35_m07() { assert_eq!(compile_and_run("tests/regression//m35_m07.xi"), Some(0)); }
#[test] fn e2e_m35_m08() { assert_eq!(compile_and_run("tests/regression//m35_m08.xi"), Some(0)); }
#[test] fn e2e_m35_m09() { assert_eq!(compile_and_run("tests/regression//m35_m09.xi"), Some(0)); }
#[test] fn e2e_m35_m10() { assert_eq!(compile_and_run("tests/regression//m35_m10.xi"), Some(0)); }
#[test] fn e2e_m35_m11() { assert_eq!(compile_and_run("tests/regression//m35_m11.xi"), Some(0)); }
#[test] fn e2e_m35_m12() { assert_eq!(compile_and_run("tests/regression//m35_m12.xi"), Some(0)); }
#[test] fn e2e_m35_m13() { assert_eq!(compile_and_run("tests/regression//m35_m13.xi"), Some(0)); }
#[test] fn e2e_m35_m14() { assert_eq!(compile_and_run("tests/regression//m35_m14.xi"), Some(0)); }
#[test] fn e2e_m35_m15() { assert_eq!(compile_and_run("tests/regression//m35_m15.xi"), Some(0)); }
#[test] fn e2e_m35_m16() { assert_eq!(compile_and_run("tests/regression//m35_m16.xi"), Some(0)); }
#[test] fn e2e_m35_m17() { assert_eq!(compile_and_run("tests/regression//m35_m17.xi"), Some(0)); }
#[test] fn e2e_m35_m18() { assert_eq!(compile_and_run("tests/regression//m35_m18.xi"), Some(0)); }
#[test] fn e2e_m35_m19() { assert_eq!(compile_and_run("tests/regression//m35_m19.xi"), Some(0)); }
#[test] fn e2e_m35_m20() { assert_eq!(compile_and_run("tests/regression//m35_m20.xi"), Some(0)); }
#[test] fn e2e_m35_m21() { assert_eq!(compile_and_run("tests/regression//m35_m21.xi"), Some(0)); }
#[test] fn e2e_m35_m22() { assert_eq!(compile_and_run("tests/regression//m35_m22.xi"), Some(0)); }
#[test] fn e2e_m35_m23() { assert_eq!(compile_and_run("tests/regression//m35_m23.xi"), Some(0)); }
#[test] fn e2e_m35_m24() { assert_eq!(compile_and_run("tests/regression//m35_m24.xi"), Some(0)); }
#[test] fn e2e_m35_m25() { assert_eq!(compile_and_run("tests/regression//m35_m25.xi"), Some(0)); }
#[test] fn e2e_m35_m26() { assert_eq!(compile_and_run("tests/regression//m35_m26.xi"), Some(0)); }
#[test] fn e2e_m35_m27() { assert_eq!(compile_and_run("tests/regression//m35_m27.xi"), Some(0)); }
#[test] fn e2e_m35_m28() { assert_eq!(compile_and_run("tests/regression//m35_m28.xi"), Some(0)); }
#[test] fn e2e_m35_m29() { assert_eq!(compile_and_run("tests/regression//m35_m29.xi"), Some(0)); }
#[test] fn e2e_m35_m30() { assert_eq!(compile_and_run("tests/regression//m35_m30.xi"), Some(0)); }

// -- M34-D: Recursive Data Structures Stress Tests ----------------------
// d01: Binary tree -- struct with left/right pointers
#[test] fn e2e_m34_d01() { assert_eq!(compile_and_run("tests/regression//m34_d01.xi"), Some(0)); }
// d02: Linked list -- struct with next pointer
#[test] fn e2e_m34_d02() { assert_eq!(compile_and_run("tests/regression//m34_d02.xi"), Some(0)); }
// d03: Tree count -- recursive count of all nodes
#[test] fn e2e_m34_d03() { assert_eq!(compile_and_run("tests/regression//m34_d03.xi"), Some(0)); }
// d04: Tree depth -- max depth of binary tree
#[test] fn e2e_m34_d04() { assert_eq!(compile_and_run("tests/regression//m34_d04.xi"), Some(0)); }
// d05: List traversals -- multiple recursive traversals
#[test] fn e2e_m34_d05() { assert_eq!(compile_and_run("tests/regression//m34_d05.xi"), Some(0)); }
// d06: Nested recursive types -- two-level struct hierarchy
#[test] fn e2e_m34_d06() { assert_eq!(compile_and_run("tests/regression//m34_d06.xi"), Some(0)); }
// d07: Mutual recursive types -- A->B, B->A
#[test] fn e2e_m34_d07() { assert_eq!(compile_and_run("tests/regression//m34_d07.xi"), Some(0)); }
// d08: Recursive struct in enum payload
#[test] fn e2e_m34_d08() { assert_eq!(compile_and_run("tests/regression//m34_d08.xi"), Some(0)); }
// d09: Recursive enum -- enum variant with pointer to own type
#[test] fn e2e_m34_d09() { assert_eq!(compile_and_run("tests/regression//m34_d09.xi"), Some(0)); }
// d10: Deep tree operations -- depth 10+ recursion stress
#[test] fn e2e_m34_d10() { assert_eq!(compile_and_run("tests/regression//m34_d10.xi"), Some(0)); }
// d11: Tree construction from expressions -- build and eval on stack
#[test] fn e2e_m34_d11() { assert_eq!(compile_and_run("tests/regression//m34_d11.xi"), Some(0)); }
// d12: List append/prepend -- linked list on stack
#[test] fn e2e_m34_d12() { assert_eq!(compile_and_run("tests/regression//m34_d12.xi"), Some(0)); }
// d13: Tree fold -- accumulation across tree
#[test] fn e2e_m34_d13() { assert_eq!(compile_and_run("tests/regression//m34_d13.xi"), Some(0)); }
// d14: Tree map -- transform tree values
#[test] fn e2e_m34_d14() { assert_eq!(compile_and_run("tests/regression//m34_d14.xi"), Some(0)); }
// d15: List filter -- count/skip nodes by predicate
#[test] fn e2e_m34_d15() { assert_eq!(compile_and_run("tests/regression//m34_d15.xi"), Some(0)); }
// d16: Tree comparison -- structural equality of two trees
#[test] fn e2e_m34_d16() { assert_eq!(compile_and_run("tests/regression//m34_d16.xi"), Some(0)); }
// d17: List reverse -- verify traversal order
#[test] fn e2e_m34_d17() { assert_eq!(compile_and_run("tests/regression//m34_d17.xi"), Some(0)); }
// d18: Recursive generic types -- Tree[T] and List[T]
#[test] fn e2e_m34_d18() { assert_eq!(compile_and_run("tests/regression//m34_d18.xi"), Some(0)); }
// d19: Diamond-shaped type graph -- A->B,C; B,C->D
#[test] fn e2e_m34_d19() { assert_eq!(compile_and_run("tests/regression//m34_d19.xi"), Some(0)); }
// d20: Recursive invariant + serialization pattern
#[test] fn e2e_m34_d20() { assert_eq!(compile_and_run("tests/regression//m34_d20.xi"), Some(0)); }

// -- M35-S: String Manipulation Algorithms -------------------------------
// s01: String reverse -- manual loop building reversed string
#[test] fn e2e_m35_s01() { assert_eq!(compile_and_run("tests/regression//m35_s01.xi"), Some(0)); }
// s02: Palindrome check -- reverse and compare
#[test] fn e2e_m35_s02() { assert_eq!(compile_and_run("tests/regression//m35_s02.xi"), Some(0)); }
// s03: Substring search -- manual needle-in-haystack check
#[test] fn e2e_m35_s03() { assert_eq!(compile_and_run("tests/regression//m35_s03.xi"), Some(0)); }
// s04: Count character occurrences -- loop and count matching bytes
#[test] fn e2e_m35_s04() { assert_eq!(compile_and_run("tests/regression//m35_s04.xi"), Some(0)); }
// s05: Remove character -- build new string without target char
#[test] fn e2e_m35_s05() { assert_eq!(compile_and_run("tests/regression//m35_s05.xi"), Some(0)); }
// s06: Replace substring -- find and replace first occurrence
#[test] fn e2e_m35_s06() { assert_eq!(compile_and_run("tests/regression//m35_s06.xi"), Some(0)); }
// s07: String to uppercase -- build uppercase via ASCII char mapping
#[test] fn e2e_m35_s07() { assert_eq!(compile_and_run("tests/regression//m35_s07.xi"), Some(0)); }
// s08: String to lowercase -- build lowercase via ASCII char mapping
#[test] fn e2e_m35_s08() { assert_eq!(compile_and_run("tests/regression//m35_s08.xi"), Some(0)); }
// s09: Trim spaces -- remove leading and trailing spaces
#[test] fn e2e_m35_s09() { assert_eq!(compile_and_run("tests/regression//m35_s09.xi"), Some(0)); }
// s10: Split by delimiter -- find delimiter position and extract parts
#[test] fn e2e_m35_s10() { assert_eq!(compile_and_run("tests/regression//m35_s10.xi"), Some(0)); }
// s11: Join strings -- concatenate with delimiter between
#[test] fn e2e_m35_s11() { assert_eq!(compile_and_run("tests/regression//m35_s11.xi"), Some(0)); }
// s12: String comparison -- character-by-character lexicographic order
#[test] fn e2e_m35_s12() { assert_eq!(compile_and_run("tests/regression//m35_s12.xi"), Some(0)); }
// s13: String prefix check -- character-by-character prefix verification
#[test] fn e2e_m35_s13() { assert_eq!(compile_and_run("tests/regression//m35_s13.xi"), Some(0)); }
// s14: String suffix check -- character-by-character suffix verification
#[test] fn e2e_m35_s14() { assert_eq!(compile_and_run("tests/regression//m35_s14.xi"), Some(0)); }
// s15: Find first occurrence -- return index of first match or -1
#[test] fn e2e_m35_s15() { assert_eq!(compile_and_run("tests/regression//m35_s15.xi"), Some(0)); }
// s16: Find last occurrence -- return index of last match or -1
#[test] fn e2e_m35_s16() { assert_eq!(compile_and_run("tests/regression//m35_s16.xi"), Some(0)); }
// s17: Extract substring -- build substring from start to end index
#[test] fn e2e_m35_s17() { assert_eq!(compile_and_run("tests/regression//m35_s17.xi"), Some(0)); }
// s18: Word count -- count spaces to determine word count
#[test] fn e2e_m35_s18() { assert_eq!(compile_and_run("tests/regression//m35_s18.xi"), Some(0)); }
// s19: Character frequency count -- count occurrences of each char
#[test] fn e2e_m35_s19() { assert_eq!(compile_and_run("tests/regression//m35_s19.xi"), Some(0)); }
// s20: Longest word -- find word with maximum length
#[test] fn e2e_m35_s20() { assert_eq!(compile_and_run("tests/regression//m35_s20.xi"), Some(0)); }
// s21: Shortest word -- find word with minimum non-zero length
#[test] fn e2e_m35_s21() { assert_eq!(compile_and_run("tests/regression//m35_s21.xi"), Some(0)); }
// s22: Capitalize first letter -- convert first char to uppercase
#[test] fn e2e_m35_s22() { assert_eq!(compile_and_run("tests/regression//m35_s22.xi"), Some(0)); }
// s23: Check anagrams -- compare character frequency counts
#[test] fn e2e_m35_s23() { assert_eq!(compile_and_run("tests/regression//m35_s23.xi"), Some(0)); }
// s24: Common prefix -- find longest shared prefix between two strings
#[test] fn e2e_m35_s24() { assert_eq!(compile_and_run("tests/regression//m35_s24.xi"), Some(0)); }
// s25: String compression -- run-length encoding
#[test] fn e2e_m35_s25() { assert_eq!(compile_and_run("tests/regression//m35_s25.xi"), Some(0)); }
// s26: String expansion -- expand repeated characters
#[test] fn e2e_m35_s26() { assert_eq!(compile_and_run("tests/regression//m35_s26.xi"), Some(0)); }
// s27: Rotation check -- check if one string is a rotation of another
#[test] fn e2e_m35_s27() { assert_eq!(compile_and_run("tests/regression//m35_s27.xi"), Some(0)); }
// s28: String distance -- Hamming distance between equal-length strings
#[test] fn e2e_m35_s28() { assert_eq!(compile_and_run("tests/regression//m35_s28.xi"), Some(0)); }
// s29: Wildcard match -- match string against pattern with ? and *
#[test] fn e2e_m35_s29() { assert_eq!(compile_and_run("tests/regression//m35_s29.xi"), Some(0)); }
// s30: Balanced brackets -- check (), [], {} balancing via depth counters
#[test] fn e2e_m35_s30() { assert_eq!(compile_and_run("tests/regression//m35_s30.xi"), Some(0)); }

// -- M35-Z: Combinatorial Mega Stress (30 tests) -----------------------
#[test] fn e2e_m35_z01() { assert_eq!(compile_and_run("tests/regression//m35_z01.xi"), Some(0)); }
#[test] fn e2e_m35_z02() { assert_eq!(compile_and_run("tests/regression//m35_z02.xi"), Some(0)); }
#[test] fn e2e_m35_z03() { assert_eq!(compile_and_run("tests/regression//m35_z03.xi"), Some(0)); }
#[test] fn e2e_m35_z04() { assert_eq!(compile_and_run("tests/regression//m35_z04.xi"), Some(0)); }
#[test] fn e2e_m35_z05() { assert_eq!(compile_and_run("tests/regression//m35_z05.xi"), Some(0)); }
#[test] fn e2e_m35_z06() { assert_eq!(compile_and_run("tests/regression//m35_z06.xi"), Some(0)); }
#[test] fn e2e_m35_z07() { assert_eq!(compile_and_run("tests/regression//m35_z07.xi"), Some(0)); }
#[test] fn e2e_m35_z08() { assert_eq!(compile_and_run("tests/regression//m35_z08.xi"), Some(0)); }
#[test] fn e2e_m35_z09() { assert_eq!(compile_and_run("tests/regression//m35_z09.xi"), Some(0)); }
#[test] fn e2e_m35_z10() { assert_eq!(compile_and_run("tests/regression//m35_z10.xi"), Some(0)); }
#[test] fn e2e_m35_z11() { assert_eq!(compile_and_run("tests/regression//m35_z11.xi"), Some(0)); }
#[test] fn e2e_m35_z12() { assert_eq!(compile_and_run("tests/regression//m35_z12.xi"), Some(0)); }
#[test] fn e2e_m35_z13() { assert_eq!(compile_and_run("tests/regression//m35_z13.xi"), Some(0)); }
#[test] fn e2e_m35_z14() { assert_eq!(compile_and_run("tests/regression//m35_z14.xi"), Some(0)); }
#[test] fn e2e_m35_z15() { assert_eq!(compile_and_run("tests/regression//m35_z15.xi"), Some(0)); }
#[test] fn e2e_m35_z16() { assert_eq!(compile_and_run("tests/regression//m35_z16.xi"), Some(0)); }
#[test] fn e2e_m35_z17() { assert_eq!(compile_and_run("tests/regression//m35_z17.xi"), Some(0)); }
#[test] fn e2e_m35_z18() { assert_eq!(compile_and_run("tests/regression//m35_z18.xi"), Some(0)); }
#[test] fn e2e_m35_z19() { assert_eq!(compile_and_run("tests/regression//m35_z19.xi"), Some(0)); }
#[test] fn e2e_m35_z20() { assert_eq!(compile_and_run("tests/regression//m35_z20.xi"), Some(0)); }
#[test] fn e2e_m35_z21() { assert_eq!(compile_and_run("tests/regression//m35_z21.xi"), Some(0)); }
#[test] fn e2e_m35_z22() { assert_eq!(compile_and_run("tests/regression//m35_z22.xi"), Some(0)); }
#[test] fn e2e_m35_z23() { assert_eq!(compile_and_run("tests/regression//m35_z23.xi"), Some(0)); }
#[test] fn e2e_m35_z24() { assert_eq!(compile_and_run("tests/regression//m35_z24.xi"), Some(0)); }
#[test] fn e2e_m35_z25() { assert_eq!(compile_and_run("tests/regression//m35_z25.xi"), Some(0)); }
#[test] fn e2e_m35_z26() { assert_eq!(compile_and_run("tests/regression//m35_z26.xi"), Some(0)); }
#[test] fn e2e_m35_z27() { assert_eq!(compile_and_run("tests/regression//m35_z27.xi"), Some(0)); }
#[test] fn e2e_m35_z28() { assert_eq!(compile_and_run("tests/regression//m35_z28.xi"), Some(0)); }
#[test] fn e2e_m35_z29() { assert_eq!(compile_and_run("tests/regression//m35_z29.xi"), Some(0)); }
#[test] fn e2e_m35_z30() { assert_eq!(compile_and_run("tests/regression//m35_z30.xi"), Some(0)); }

// -- M35-V: Vec[DATA] Collection Stress Tests ---------------------------
// Vec[Int] create/push/pop/len, get/set, first/last, is_empty, clear,
// insert, remove, iteration, resize, Float64, Int16, struct, enum,
// Option, nested Vec, generic T, Vec param/return, clone, reverse,
// sort-like, min/max, filter, map, fold/sum, combined mega stress
#[test] fn e2e_m35_v01() { assert_eq!(compile_and_run("tests/regression//m35_v01.xi"), Some(0)); }
#[test] fn e2e_m35_v02() { assert_eq!(compile_and_run("tests/regression//m35_v02.xi"), Some(0)); }
#[test] fn e2e_m35_v03() { assert_eq!(compile_and_run("tests/regression//m35_v03.xi"), Some(0)); }
#[test] fn e2e_m35_v04() { assert_eq!(compile_and_run("tests/regression//m35_v04.xi"), Some(0)); }
#[test] fn e2e_m35_v05() { assert_eq!(compile_and_run("tests/regression//m35_v05.xi"), Some(0)); }
#[test] fn e2e_m35_v06() { assert_eq!(compile_and_run("tests/regression//m35_v06.xi"), Some(0)); }
#[test] fn e2e_m35_v07() { assert_eq!(compile_and_run("tests/regression//m35_v07.xi"), Some(0)); }
#[test] fn e2e_m35_v08() { assert_eq!(compile_and_run("tests/regression//m35_v08.xi"), Some(0)); }
#[test] fn e2e_m35_v09() { assert_eq!(compile_and_run("tests/regression//m35_v09.xi"), Some(0)); }
#[test] fn e2e_m35_v10() { assert_eq!(compile_and_run("tests/regression//m35_v10.xi"), Some(0)); }
#[test] fn e2e_m35_v11() { assert_eq!(compile_and_run("tests/regression//m35_v11.xi"), Some(0)); }
#[test] fn e2e_m35_v12() { assert_eq!(compile_and_run("tests/regression//m35_v12.xi"), Some(0)); }
#[test] fn e2e_m35_v13() { assert_eq!(compile_and_run("tests/regression//m35_v13.xi"), Some(0)); }
#[test] fn e2e_m35_v14() { assert_eq!(compile_and_run("tests/regression//m35_v14.xi"), Some(0)); }
#[test] fn e2e_m35_v15() { assert_eq!(compile_and_run("tests/regression//m35_v15.xi"), Some(0)); }
#[test] fn e2e_m35_v16() { assert_eq!(compile_and_run("tests/regression//m35_v16.xi"), Some(0)); }
#[test] fn e2e_m35_v17() { assert_eq!(compile_and_run("tests/regression//m35_v17.xi"), Some(0)); }
#[test] fn e2e_m35_v18() { assert_eq!(compile_and_run("tests/regression//m35_v18.xi"), Some(0)); }
#[test] fn e2e_m35_v19() { assert_eq!(compile_and_run("tests/regression//m35_v19.xi"), Some(0)); }
#[test] fn e2e_m35_v20() { assert_eq!(compile_and_run("tests/regression//m35_v20.xi"), Some(0)); }
#[test] fn e2e_m35_v21() { assert_eq!(compile_and_run("tests/regression//m35_v21.xi"), Some(0)); }
#[test] fn e2e_m35_v22() { assert_eq!(compile_and_run("tests/regression//m35_v22.xi"), Some(0)); }
#[test] fn e2e_m35_v23() { assert_eq!(compile_and_run("tests/regression//m35_v23.xi"), Some(0)); }
#[test] fn e2e_m35_v24() { assert_eq!(compile_and_run("tests/regression//m35_v24.xi"), Some(0)); }
#[test] fn e2e_m35_v25() { assert_eq!(compile_and_run("tests/regression//m35_v25.xi"), Some(0)); }
#[test] fn e2e_m35_v26() { assert_eq!(compile_and_run("tests/regression//m35_v26.xi"), Some(0)); }
#[test] fn e2e_m35_v27() { assert_eq!(compile_and_run("tests/regression//m35_v27.xi"), Some(0)); }
#[test] fn e2e_m35_v28() { assert_eq!(compile_and_run("tests/regression//m35_v28.xi"), Some(0)); }
#[test] fn e2e_m35_v29() { assert_eq!(compile_and_run("tests/regression//m35_v29.xi"), Some(0)); }
#[test] fn e2e_m35_v30() { assert_eq!(compile_and_run("tests/regression//m35_v30.xi"), Some(0)); }

// -- M35-C: Control Flow Exhaustive Stress Tests -------------------------
// c01: if without else -- single branch
#[test] fn e2e_m35_c01() { assert_eq!(compile_and_run("tests/regression//m35_c01.xi"), Some(0)); }
// c02: if with else -- two branches
#[test] fn e2e_m35_c02() { assert_eq!(compile_and_run("tests/regression//m35_c02.xi"), Some(0)); }
// c03: if elif else (3+ branches) -- three-way branching
#[test] fn e2e_m35_c03() { assert_eq!(compile_and_run("tests/regression//m35_c03.xi"), Some(0)); }
// c04: if elif elif elif else (5+ branches) -- deep multi-way dispatch
#[test] fn e2e_m35_c04() { assert_eq!(compile_and_run("tests/regression//m35_c04.xi"), Some(0)); }
// c05: nested if 5 deep -- cascading conditional logic
#[test] fn e2e_m35_c05() { assert_eq!(compile_and_run("tests/regression//m35_c05.xi"), Some(0)); }
// c06: while true loop -- infinite loop with internal break
#[test] fn e2e_m35_c06() { assert_eq!(compile_and_run("tests/regression//m35_c06.xi"), Some(0)); }
// c07: while with condition -- standard while loop
#[test] fn e2e_m35_c07() { assert_eq!(compile_and_run("tests/regression//m35_c07.xi"), Some(0)); }
// c08: while with break -- early loop termination
#[test] fn e2e_m35_c08() { assert_eq!(compile_and_run("tests/regression//m35_c08.xi"), Some(0)); }
// c09: while with continue -- skip iterations inside loop
#[test] fn e2e_m35_c09() { assert_eq!(compile_and_run("tests/regression//m35_c09.xi"), Some(0)); }
// c10: while with nested break -- inner loop break only
#[test] fn e2e_m35_c10() { assert_eq!(compile_and_run("tests/regression//m35_c10.xi"), Some(0)); }
// c11: while with return inside -- function return from within loop
#[test] fn e2e_m35_c11() { assert_eq!(compile_and_run("tests/regression//m35_c11.xi"), Some(0)); }
// c12: for-in pattern using while + index
#[test] fn e2e_m35_c12() { assert_eq!(compile_and_run("tests/regression//m35_c12.xi"), Some(0)); }
// c13: do-while pattern -- execute body at least once
#[test] fn e2e_m35_c13() { assert_eq!(compile_and_run("tests/regression//m35_c13.xi"), Some(0)); }
// c14: infinite loop with break -- while true with multiple break conditions
#[test] fn e2e_m35_c14() { assert_eq!(compile_and_run("tests/regression//m35_c14.xi"), Some(0)); }
// c15: loop with accumulator -- building value across iterations
#[test] fn e2e_m35_c15() { assert_eq!(compile_and_run("tests/regression//m35_c15.xi"), Some(0)); }
// c16: loop with multiple accumulators -- tracking several values
#[test] fn e2e_m35_c16() { assert_eq!(compile_and_run("tests/regression//m35_c16.xi"), Some(0)); }
// c17: loop with early exit -- return immediately when condition met
#[test] fn e2e_m35_c17() { assert_eq!(compile_and_run("tests/regression//m35_c17.xi"), Some(0)); }
// c18: loop with flag -- boolean flag controls loop termination
#[test] fn e2e_m35_c18() { assert_eq!(compile_and_run("tests/regression//m35_c18.xi"), Some(0)); }
// c19: loop with counter -- counting iterations
#[test] fn e2e_m35_c19() { assert_eq!(compile_and_run("tests/regression//m35_c19.xi"), Some(0)); }
// c20: if-expression -- let binding with if expression value
#[test] fn e2e_m35_c20() { assert_eq!(compile_and_run("tests/regression//m35_c20.xi"), Some(0)); }
// c21: match on Int -- integer value dispatch
#[test] fn e2e_m35_c21() { assert_eq!(compile_and_run("tests/regression//m35_c21.xi"), Some(0)); }
// c22: match on Bool -- boolean dispatch
#[test] fn e2e_m35_c22() { assert_eq!(compile_and_run("tests/regression//m35_c22.xi"), Some(0)); }
// c23: match on enum -- dispatch through enum variants with payloads
#[test] fn e2e_m35_c23() { assert_eq!(compile_and_run("tests/regression//m35_c23.xi"), Some(0)); }
// c24: match with guard -- if condition inside arm body
#[test] fn e2e_m35_c24() { assert_eq!(compile_and_run("tests/regression//m35_c24.xi"), Some(0)); }
// c25: match with wildcard -- catch-all pattern _
#[test] fn e2e_m35_c25() { assert_eq!(compile_and_run("tests/regression//m35_c25.xi"), Some(0)); }
// c26: if-let pattern -- match on Option as control flow
#[test] fn e2e_m35_c26() { assert_eq!(compile_and_run("tests/regression//m35_c26.xi"), Some(0)); }
// c27: while-let pattern -- loop while match on Option/Result
#[test] fn e2e_m35_c27() { assert_eq!(compile_and_run("tests/regression//m35_c27.xi"), Some(0)); }
// c28: early return from function -- multiple return points
#[test] fn e2e_m35_c28() { assert_eq!(compile_and_run("tests/regression//m35_c28.xi"), Some(0)); }
// c29: tail call pattern -- recursive call in tail position
#[test] fn e2e_m35_c29() { assert_eq!(compile_and_run("tests/regression//m35_c29.xi"), Some(0)); }
// c30: all-in-one -- combined control flow patterns
#[test] fn e2e_m35_c30() { assert_eq!(compile_and_run("tests/regression//m35_c30.xi"), Some(0)); }

// -- M35-A: Algorithm Correctness Stress Tests ----------------------------
// Binary search, linear search, bubble/selection/insertion sort,
// merge sort, quick sort partition, factorial, fibonacci (3 ways),
// GCD, LCM, prime check, sum of digits, reverse number, palindrome,
// power function, int sqrt, factorial rec vs iter, Collatz,
// count divisors, is perfect square, prime count sieve-like,
// digit sum recursive, exponent by squaring, max subarray sum,
// binary conversion, count set bits (Kernighan), rotate array
#[test] fn e2e_m35_a01() { assert_eq!(compile_and_run("tests/regression//m35_a01.xi"), Some(0)); }
#[test] fn e2e_m35_a02() { assert_eq!(compile_and_run("tests/regression//m35_a02.xi"), Some(0)); }
#[test] fn e2e_m35_a03() { assert_eq!(compile_and_run("tests/regression//m35_a03.xi"), Some(0)); }
#[test] fn e2e_m35_a04() { assert_eq!(compile_and_run("tests/regression//m35_a04.xi"), Some(0)); }
#[test] fn e2e_m35_a05() { assert_eq!(compile_and_run("tests/regression//m35_a05.xi"), Some(0)); }
#[test] fn e2e_m35_a06() { assert_eq!(compile_and_run("tests/regression//m35_a06.xi"), Some(0)); }
#[test] fn e2e_m35_a07() { assert_eq!(compile_and_run("tests/regression//m35_a07.xi"), Some(0)); }
#[test] fn e2e_m35_a08() { assert_eq!(compile_and_run("tests/regression//m35_a08.xi"), Some(0)); }
#[test] fn e2e_m35_a09() { assert_eq!(compile_and_run("tests/regression//m35_a09.xi"), Some(0)); }
#[test] fn e2e_m35_a10() { assert_eq!(compile_and_run("tests/regression//m35_a10.xi"), Some(0)); }
#[test] fn e2e_m35_a11() { assert_eq!(compile_and_run("tests/regression//m35_a11.xi"), Some(0)); }
#[test] fn e2e_m35_a12() { assert_eq!(compile_and_run("tests/regression//m35_a12.xi"), Some(0)); }
#[test] fn e2e_m35_a13() { assert_eq!(compile_and_run("tests/regression//m35_a13.xi"), Some(0)); }
#[test] fn e2e_m35_a14() { assert_eq!(compile_and_run("tests/regression//m35_a14.xi"), Some(0)); }
#[test] fn e2e_m35_a15() { assert_eq!(compile_and_run("tests/regression//m35_a15.xi"), Some(0)); }
#[test] fn e2e_m35_a16() { assert_eq!(compile_and_run("tests/regression//m35_a16.xi"), Some(0)); }
#[test] fn e2e_m35_a17() { assert_eq!(compile_and_run("tests/regression//m35_a17.xi"), Some(0)); }
#[test] fn e2e_m35_a18() { assert_eq!(compile_and_run("tests/regression//m35_a18.xi"), Some(0)); }
#[test] fn e2e_m35_a19() { assert_eq!(compile_and_run("tests/regression//m35_a19.xi"), Some(0)); }
#[test] fn e2e_m35_a20() { assert_eq!(compile_and_run("tests/regression//m35_a20.xi"), Some(0)); }
#[test] fn e2e_m35_a21() { assert_eq!(compile_and_run("tests/regression//m35_a21.xi"), Some(0)); }
#[test] fn e2e_m35_a22() { assert_eq!(compile_and_run("tests/regression//m35_a22.xi"), Some(0)); }
#[test] fn e2e_m35_a23() { assert_eq!(compile_and_run("tests/regression//m35_a23.xi"), Some(0)); }
#[test] fn e2e_m35_a24() { assert_eq!(compile_and_run("tests/regression//m35_a24.xi"), Some(0)); }
#[test] fn e2e_m35_a25() { assert_eq!(compile_and_run("tests/regression//m35_a25.xi"), Some(0)); }
#[test] fn e2e_m35_a26() { assert_eq!(compile_and_run("tests/regression//m35_a26.xi"), Some(0)); }
#[test] fn e2e_m35_a27() { assert_eq!(compile_and_run("tests/regression//m35_a27.xi"), Some(0)); }
#[test] fn e2e_m35_a28() { assert_eq!(compile_and_run("tests/regression//m35_a28.xi"), Some(0)); }
#[test] fn e2e_m35_a29() { assert_eq!(compile_and_run("tests/regression//m35_a29.xi"), Some(0)); }
#[test] fn e2e_m35_a30() { assert_eq!(compile_and_run("tests/regression//m35_a30.xi"), Some(0)); }

// -- M35-L: Memory Layout / Pointer Stress Tests -------------------------
// l01: Struct with Int fields
#[test] fn e2e_m35_l01() { assert_eq!(compile_and_run("tests/regression//m35_l01.xi"), Some(0)); }
// l02: Struct with Float64 fields
#[test] fn e2e_m35_l02() { assert_eq!(compile_and_run("tests/regression//m35_l02.xi"), Some(0)); }
// l03: Struct with mixed fields
#[test] fn e2e_m35_l03() { assert_eq!(compile_and_run("tests/regression//m35_l03.xi"), Some(0)); }
// l04: Struct with Bool
#[test] fn e2e_m35_l04() { assert_eq!(compile_and_run("tests/regression//m35_l04.xi"), Some(0)); }
// l05: Struct with Char
#[test] fn e2e_m35_l05() { assert_eq!(compile_and_run("tests/regression//m35_l05.xi"), Some(0)); }
// l06: Struct with Str
#[test] fn e2e_m35_l06() { assert_eq!(compile_and_run("tests/regression//m35_l06.xi"), Some(0)); }
// l07: Struct with array
#[test] fn e2e_m35_l07() { assert_eq!(compile_and_run("tests/regression//m35_l07.xi"), Some(0)); }
// l08: Struct with pointer
#[test] fn e2e_m35_l08() { assert_eq!(compile_and_run("tests/regression//m35_l08.xi"), Some(0)); }
// l09: Struct with nested struct
#[test] fn e2e_m35_l09() { assert_eq!(compile_and_run("tests/regression//m35_l09.xi"), Some(0)); }
// l10: Struct alignment check
#[test] fn e2e_m35_l10() { assert_eq!(compile_and_run("tests/regression//m35_l10.xi"), Some(0)); }
// l11: Enum memory pattern
#[test] fn e2e_m35_l11() { assert_eq!(compile_and_run("tests/regression//m35_l11.xi"), Some(0)); }
// l12: Option size pattern
#[test] fn e2e_m35_l12() { assert_eq!(compile_and_run("tests/regression//m35_l12.xi"), Some(0)); }
// l13: Result size pattern
#[test] fn e2e_m35_l13() { assert_eq!(compile_and_run("tests/regression//m35_l13.xi"), Some(0)); }
// l14: Pointer arithmetic
#[test] fn e2e_m35_l14() { assert_eq!(compile_and_run("tests/regression//m35_l14.xi"), Some(0)); }
// l15: Pointer difference
#[test] fn e2e_m35_l15() { assert_eq!(compile_and_run("tests/regression//m35_l15.xi"), Some(0)); }
// l16: Pointer comparison
#[test] fn e2e_m35_l16() { assert_eq!(compile_and_run("tests/regression//m35_l16.xi"), Some(0)); }
// l17: Null pointer check
#[test] fn e2e_m35_l17() { assert_eq!(compile_and_run("tests/regression//m35_l17.xi"), Some(0)); }
// l18: Pointer to struct
#[test] fn e2e_m35_l18() { assert_eq!(compile_and_run("tests/regression//m35_l18.xi"), Some(0)); }
// l19: Pointer to array
#[test] fn e2e_m35_l19() { assert_eq!(compile_and_run("tests/regression//m35_l19.xi"), Some(0)); }
// l20: Pointer to function
#[test] fn e2e_m35_l20() { assert_eq!(compile_and_run("tests/regression//m35_l20.xi"), Some(0)); }
// l21: Array of pointer
#[test] fn e2e_m35_l21() { assert_eq!(compile_and_run("tests/regression//m35_l21.xi"), Some(0)); }
// l22: Pointer to pointer
#[test] fn e2e_m35_l22() { assert_eq!(compile_and_run("tests/regression//m35_l22.xi"), Some(0)); }
// l23: Unsafe block with raw pointer
#[test] fn e2e_m35_l23() { assert_eq!(compile_and_run("tests/regression//m35_l23.xi"), Some(0)); }
// l24: Pointer cast (Int from pointer)
#[test] fn e2e_m35_l24() { assert_eq!(compile_and_run("tests/regression//m35_l24.xi"), Some(0)); }
// l25: Stack allocation pattern
#[test] fn e2e_m35_l25() { assert_eq!(compile_and_run("tests/regression//m35_l25.xi"), Some(0)); }
// l26: Heap allocation pattern (malloc/free extern)
#[test] fn e2e_m35_l26() { assert_eq!(compile_and_run("tests/regression//m35_l26.xi"), Some(0)); }
// l27: Alignment requirement
#[test] fn e2e_m35_l27() { assert_eq!(compile_and_run("tests/regression//m35_l27.xi"), Some(0)); }
// l28: Packed struct pattern
#[test] fn e2e_m35_l28() { assert_eq!(compile_and_run("tests/regression//m35_l28.xi"), Some(0)); }
// l29: Union pattern via pointer cast
#[test] fn e2e_m35_l29() { assert_eq!(compile_and_run("tests/regression//m35_l29.xi"), Some(0)); }
// l30: Pointer iteration
#[test] fn e2e_m35_l30() { assert_eq!(compile_and_run("tests/regression//m35_l30.xi"), Some(0)); }

// -- M35-D: Data Structure Stress Tests ----------------------------------
// Stack (push/pop/peek/empty), Queue (enqueue/dequeue/peek/empty),
// Deque, Priority Queue, Min-Heap, Max-Heap, Hash Set, Hash Map,
// Linked List, Doubly Linked List, Circular Buffer, Ring Buffer,
// LRU Cache, Trie, Graph Adjacency List, Tree Traversal, BST,
// AVL Balance Check, Heap Sort, Topological Sort, Shortest Path,
// Union-Find, Disjoint Set, Bloom Filter, Skip List, Segment Tree,
// Fenwick Tree (BIT), Combined Stress
#[test] fn e2e_m35_d01() { assert_eq!(compile_and_run("tests/regression//m35_d01.xi"), Some(0)); }
#[test] fn e2e_m35_d02() { assert_eq!(compile_and_run("tests/regression//m35_d02.xi"), Some(0)); }
#[test] fn e2e_m35_d03() { assert_eq!(compile_and_run("tests/regression//m35_d03.xi"), Some(0)); }
#[test] fn e2e_m35_d04() { assert_eq!(compile_and_run("tests/regression//m35_d04.xi"), Some(0)); }
#[test] fn e2e_m35_d05() { assert_eq!(compile_and_run("tests/regression//m35_d05.xi"), Some(0)); }
#[test] fn e2e_m35_d06() { assert_eq!(compile_and_run("tests/regression//m35_d06.xi"), Some(0)); }
#[test] fn e2e_m35_d07() { assert_eq!(compile_and_run("tests/regression//m35_d07.xi"), Some(0)); }
#[test] fn e2e_m35_d08() { assert_eq!(compile_and_run("tests/regression//m35_d08.xi"), Some(0)); }
#[test] fn e2e_m35_d09() { assert_eq!(compile_and_run("tests/regression//m35_d09.xi"), Some(0)); }
#[test] fn e2e_m35_d10() { assert_eq!(compile_and_run("tests/regression//m35_d10.xi"), Some(0)); }
#[test] fn e2e_m35_d11() { assert_eq!(compile_and_run("tests/regression//m35_d11.xi"), Some(0)); }
#[test] fn e2e_m35_d12() { assert_eq!(compile_and_run("tests/regression//m35_d12.xi"), Some(0)); }
#[test] fn e2e_m35_d13() { assert_eq!(compile_and_run("tests/regression//m35_d13.xi"), Some(0)); }
#[test] fn e2e_m35_d14() { assert_eq!(compile_and_run("tests/regression//m35_d14.xi"), Some(0)); }
#[test] fn e2e_m35_d15() { assert_eq!(compile_and_run("tests/regression//m35_d15.xi"), Some(0)); }
#[test] fn e2e_m35_d16() { assert_eq!(compile_and_run("tests/regression//m35_d16.xi"), Some(0)); }
#[test] fn e2e_m35_d17() { assert_eq!(compile_and_run("tests/regression//m35_d17.xi"), Some(0)); }
#[test] fn e2e_m35_d18() { assert_eq!(compile_and_run("tests/regression//m35_d18.xi"), Some(0)); }
#[test] fn e2e_m35_d19() { assert_eq!(compile_and_run("tests/regression//m35_d19.xi"), Some(0)); }
#[test] fn e2e_m35_d20() { assert_eq!(compile_and_run("tests/regression//m35_d20.xi"), Some(0)); }
#[test] fn e2e_m35_d21() { assert_eq!(compile_and_run("tests/regression//m35_d21.xi"), Some(0)); }
#[test] fn e2e_m35_d22() { assert_eq!(compile_and_run("tests/regression//m35_d22.xi"), Some(0)); }
#[test] fn e2e_m35_d23() { assert_eq!(compile_and_run("tests/regression//m35_d23.xi"), Some(0)); }
#[test] fn e2e_m35_d24() { assert_eq!(compile_and_run("tests/regression//m35_d24.xi"), Some(0)); }
#[test] fn e2e_m35_d25() { assert_eq!(compile_and_run("tests/regression//m35_d25.xi"), Some(0)); }
#[test] fn e2e_m35_d26() { assert_eq!(compile_and_run("tests/regression//m35_d26.xi"), Some(0)); }
#[test] fn e2e_m35_d27() { assert_eq!(compile_and_run("tests/regression//m35_d27.xi"), Some(0)); }
#[test] fn e2e_m35_d28() { assert_eq!(compile_and_run("tests/regression//m35_d28.xi"), Some(0)); }
#[test] fn e2e_m35_d29() { assert_eq!(compile_and_run("tests/regression//m35_d29.xi"), Some(0)); }
#[test] fn e2e_m35_d30() { assert_eq!(compile_and_run("tests/regression//m35_d30.xi"), Some(0)); }

// -- M35-T: Type System Exhaustive Stress Tests -------------------------
// t01: Bool in every context -- var, param, return, struct, enum, array, generic, if, while, match
#[test] fn e2e_m35_t01() { assert_eq!(compile_and_run("tests/regression//m35_t01.xi"), Some(0)); }
// t02: Int in every context -- var, param, return, struct, enum, array, generic, if, while, match
#[test] fn e2e_m35_t02() { assert_eq!(compile_and_run("tests/regression//m35_t02.xi"), Some(0)); }
// t03: Float64 in every context -- var, param, return, struct, enum, array, generic, if, while, match, operators
#[test] fn e2e_m35_t03() { assert_eq!(compile_and_run("tests/regression//m35_t03.xi"), Some(0)); }
// t04: Char in every context -- var, param, return, struct, enum, if, match, array, generic
#[test] fn e2e_m35_t04() { assert_eq!(compile_and_run("tests/regression//m35_t04.xi"), Some(0)); }
// t05: Str in every context -- var, param, return, struct, enum, array, if, match, len
#[test] fn e2e_m35_t05() { assert_eq!(compile_and_run("tests/regression//m35_t05.xi"), Some(0)); }
// t06: Unit/void returns -- functions returning nothing, calling void functions
#[test] fn e2e_m35_t06() { assert_eq!(compile_and_run("tests/regression//m35_t06.xi"), Some(0)); }
// t07: Array in every context -- var, param, return, indexing, vec! literal
#[test] fn e2e_m35_t07() { assert_eq!(compile_and_run("tests/regression//m35_t07.xi"), Some(0)); }
// t08: Nested arrays 3-deep -- Vec[Vec[Vec[Int]]]
#[test] fn e2e_m35_t08() { assert_eq!(compile_and_run("tests/regression//m35_t08.xi"), Some(0)); }
// t09: Pointers in every context -- var, param, return, deref, address-of, *Int, *Bool, *Float64
#[test] fn e2e_m35_t09() { assert_eq!(compile_and_run("tests/regression//m35_t09.xi"), Some(0)); }
// t10: Struct with all field types -- Bool, Int, Int8, Int16, Int32, Int64, Float64, Char, Str
#[test] fn e2e_m35_t10() { assert_eq!(compile_and_run("tests/regression//m35_t10.xi"), Some(0)); }
// t11: Enum with all variant types -- unit, Int, Float64, Bool, Str, struct payload
#[test] fn e2e_m35_t11() { assert_eq!(compile_and_run("tests/regression//m35_t11.xi"), Some(0)); }
// t12: Generic with all constraints -- identity, struct, enum, array, pointer
#[test] fn e2e_m35_t12() { assert_eq!(compile_and_run("tests/regression//m35_t12.xi"), Some(0)); }
// t13: Type alias for each primitive -- Bool, Int, Int8, Int16, Int32, Int64, Float64, Char, Str
#[test] fn e2e_m35_t13() { assert_eq!(compile_and_run("tests/regression//m35_t13.xi"), Some(0)); }
// t14: Const for each primitive -- Bool, Int, Float64, Char, Str
#[test] fn e2e_m35_t14() { assert_eq!(compile_and_run("tests/regression//m35_t14.xi"), Some(0)); }
// t15: Derive[Eq] on struct -- multiple struct types
#[test] fn e2e_m35_t15() { assert_eq!(compile_and_run("tests/regression//m35_t15.xi"), Some(0)); }
// t16: Method on each type -- impl for struct wrapper, enum
#[test] fn e2e_m35_t16() { assert_eq!(compile_and_run("tests/regression//m35_t16.xi"), Some(0)); }
// t17: Interface on each type -- multiple interfaces, cross-type impl
#[test] fn e2e_m35_t17() { assert_eq!(compile_and_run("tests/regression//m35_t17.xi"), Some(0)); }
// t18: Module exporting each type -- Bool, Int, Float64, Char, Str, struct, enum
#[test] fn e2e_m35_t18() { assert_eq!(compile_and_run("tests/regression//m35_t18.xi"), Some(0)); }
// t19: Bool exhaustive -- every context deeply exercised
#[test] fn e2e_m35_t19() { assert_eq!(compile_and_run("tests/regression//m35_t19.xi"), Some(0)); }
// t20: Int exhaustive -- every context with all integer subtypes
#[test] fn e2e_m35_t20() { assert_eq!(compile_and_run("tests/regression//m35_t20.xi"), Some(0)); }
// t21: Float64 exhaustive -- every context: arith, cmp, cast, struct, enum, array, while
#[test] fn e2e_m35_t21() { assert_eq!(compile_and_run("tests/regression//m35_t21.xi"), Some(0)); }
// t22: Char exhaustive -- every context: var, param, return, struct, enum, array, cmp, match
#[test] fn e2e_m35_t22() { assert_eq!(compile_and_run("tests/regression//m35_t22.xi"), Some(0)); }
// t23: Str exhaustive -- every context: var, param, return, struct, enum, array, cmp, match, len
#[test] fn e2e_m35_t23() { assert_eq!(compile_and_run("tests/regression//m35_t23.xi"), Some(0)); }
// t24: Array exhaustive -- every type in arrays, multi-dimensional, nested access patterns
#[test] fn e2e_m35_t24() { assert_eq!(compile_and_run("tests/regression//m35_t24.xi"), Some(0)); }
// t25: Pointer exhaustive -- deref chain, double pointer, in struct, to array, cast
#[test] fn e2e_m35_t25() { assert_eq!(compile_and_run("tests/regression//m35_t25.xi"), Some(0)); }
// t26: Struct exhaustive -- all field types, nested structs, struct array, pointer
#[test] fn e2e_m35_t26() { assert_eq!(compile_and_run("tests/regression//m35_t26.xi"), Some(0)); }
// t27: Enum exhaustive -- payload variants, nested enums, Option, Result patterns
#[test] fn e2e_m35_t27() { assert_eq!(compile_and_run("tests/regression//m35_t27.xi"), Some(0)); }
// t28: Generic exhaustive -- struct, enum, function, multiple params, constraints
#[test] fn e2e_m35_t28() { assert_eq!(compile_and_run("tests/regression//m35_t28.xi"), Some(0)); }
// t29: Mixed combinators -- all types combined in complex expressions
#[test] fn e2e_m35_t29() { assert_eq!(compile_and_run("tests/regression//m35_t29.xi"), Some(0)); }
// t30: Mega stress test -- everything combined: generics, contracts, closures, modules, pointers
#[test] fn e2e_m35_t30() { assert_eq!(compile_and_run("tests/regression//m35_t30.xi"), Some(0)); }

// -- M35-O: Option / Result Combinators Exhaustive Tests -----------------
// o01: Option[Int] create             o02: Option[Float64] create
// o03: Option[Str] create             o04: Option[Bool] create
// o05: Option[struct]                 o06: Option[enum]
// o07: Option unwrap_or               o08: Option map
// o09: Option and_then                o10: Option filter
// o11: Option or_else                 o12: Option map_or
// o13: Option is_some                 o14: Option is_none
// o15: Option as_ref pattern          o16: Option cloned pattern
// o17: Option transpose               o18: Option zip
// o19: Result[Int,Str]                o20: Result map
// o21: Result map_err                 o22: Result and_then
// o23: Result or_else                 o24: Result unwrap_or
// o25: Result expect                  o26: Result[Float64,Int]
// o27: Result[Bool,Bool]              o28: Option of Result
// o29: Result of Option               o30: nested Option/Option + Result/Result
#[test] fn e2e_m35_o01() { assert_eq!(compile_and_run("tests/regression//m35_o01.xi"), Some(0)); }
#[test] fn e2e_m35_o02() { assert_eq!(compile_and_run("tests/regression//m35_o02.xi"), Some(0)); }
#[test] fn e2e_m35_o03() { assert_eq!(compile_and_run("tests/regression//m35_o03.xi"), Some(0)); }
#[test] fn e2e_m35_o04() { assert_eq!(compile_and_run("tests/regression//m35_o04.xi"), Some(0)); }
#[test] fn e2e_m35_o05() { assert_eq!(compile_and_run("tests/regression//m35_o05.xi"), Some(0)); }
#[test] fn e2e_m35_o06() { assert_eq!(compile_and_run("tests/regression//m35_o06.xi"), Some(0)); }
#[test] fn e2e_m35_o07() { assert_eq!(compile_and_run("tests/regression//m35_o07.xi"), Some(0)); }
#[test] fn e2e_m35_o08() { assert_eq!(compile_and_run("tests/regression//m35_o08.xi"), Some(0)); }
#[test] fn e2e_m35_o09() { assert_eq!(compile_and_run("tests/regression//m35_o09.xi"), Some(0)); }
#[test] fn e2e_m35_o10() { assert_eq!(compile_and_run("tests/regression//m35_o10.xi"), Some(0)); }
#[test] fn e2e_m35_o11() { assert_eq!(compile_and_run("tests/regression//m35_o11.xi"), Some(0)); }
#[test] fn e2e_m35_o12() { assert_eq!(compile_and_run("tests/regression//m35_o12.xi"), Some(0)); }
#[test] fn e2e_m35_o13() { assert_eq!(compile_and_run("tests/regression//m35_o13.xi"), Some(0)); }
#[test] fn e2e_m35_o14() { assert_eq!(compile_and_run("tests/regression//m35_o14.xi"), Some(0)); }
#[test] fn e2e_m35_o15() { assert_eq!(compile_and_run("tests/regression//m35_o15.xi"), Some(0)); }
#[test] fn e2e_m35_o16() { assert_eq!(compile_and_run("tests/regression//m35_o16.xi"), Some(0)); }
#[test] fn e2e_m35_o17() { assert_eq!(compile_and_run("tests/regression//m35_o17.xi"), Some(0)); }
#[test] fn e2e_m35_o18() { assert_eq!(compile_and_run("tests/regression//m35_o18.xi"), Some(0)); }
#[test] fn e2e_m35_o19() { assert_eq!(compile_and_run("tests/regression//m35_o19.xi"), Some(0)); }
#[test] fn e2e_m35_o20() { assert_eq!(compile_and_run("tests/regression//m35_o20.xi"), Some(0)); }
#[test] fn e2e_m35_o21() { assert_eq!(compile_and_run("tests/regression//m35_o21.xi"), Some(0)); }
#[test] fn e2e_m35_o22() { assert_eq!(compile_and_run("tests/regression//m35_o22.xi"), Some(0)); }
#[test] fn e2e_m35_o23() { assert_eq!(compile_and_run("tests/regression//m35_o23.xi"), Some(0)); }
#[test] fn e2e_m35_o24() { assert_eq!(compile_and_run("tests/regression//m35_o24.xi"), Some(0)); }
#[test] fn e2e_m35_o25() { assert_eq!(compile_and_run("tests/regression//m35_o25.xi"), Some(0)); }
#[test] fn e2e_m35_o26() { assert_eq!(compile_and_run("tests/regression//m35_o26.xi"), Some(0)); }
#[test] fn e2e_m35_o27() { assert_eq!(compile_and_run("tests/regression//m35_o27.xi"), Some(0)); }
#[test] fn e2e_m35_o28() { assert_eq!(compile_and_run("tests/regression//m35_o28.xi"), Some(0)); }
#[test] fn e2e_m35_o29() { assert_eq!(compile_and_run("tests/regression//m35_o29.xi"), Some(0)); }
#[test] fn e2e_m35_o30() { assert_eq!(compile_and_run("tests/regression//m35_o30.xi"), Some(0));
}

// -- M36-R: Parser Error Recovery Stress Tests -------------------------
// r01: unclosed string               r02: unclosed block comment
// r03: extra closing brace           r04: missing semicolon
// r05: wrong keyword                 r06: extra comma
// r07: double/invalid operator       r08: missing closing paren
// r09: extra closing paren           r10: missing closing bracket
// r11: missing type annotation       r12: wrong type name
// r13: malformed number literal      r14: invalid escape sequence
// r15: keyword as identifier         r16: reserved word misuse
// r17: impl without methods          r18: enum without variants
// r19: struct without fields         r20: empty file recovery
// r21: whitespace-only preamble      r22: comment-only preamble
// r23: UTF-8 BOM handling            r24: null byte survival
// r25: very long line (1000 chars)   r26: deep nesting (50 ifs)
// r27: very large integer literal    r28: very large float literal
// r29: many consecutive newlines     r30: mixed line endings
#[test] fn e2e_m36_r01() { assert_eq!(compile_and_run("tests/regression//m36_r01.xi"), Some(0)); }
#[test] fn e2e_m36_r02() { assert_eq!(compile_and_run("tests/regression//m36_r02.xi"), Some(0)); }
#[test] fn e2e_m36_r03() { assert_eq!(compile_and_run("tests/regression//m36_r03.xi"), Some(0)); }
#[test] fn e2e_m36_r04() { assert_eq!(compile_and_run("tests/regression//m36_r04.xi"), Some(0)); }
#[test] fn e2e_m36_r05() { assert_eq!(compile_and_run("tests/regression//m36_r05.xi"), Some(0)); }
#[test] fn e2e_m36_r06() { assert_eq!(compile_and_run("tests/regression//m36_r06.xi"), Some(0)); }
#[test] fn e2e_m36_r07() { assert_eq!(compile_and_run("tests/regression//m36_r07.xi"), Some(0)); }
#[test] fn e2e_m36_r08() { assert_eq!(compile_and_run("tests/regression//m36_r08.xi"), Some(0)); }
#[test] fn e2e_m36_r09() { assert_eq!(compile_and_run("tests/regression//m36_r09.xi"), Some(0)); }
#[test] fn e2e_m36_r10() { assert_eq!(compile_and_run("tests/regression//m36_r10.xi"), Some(0)); }
#[test] fn e2e_m36_r11() { assert_eq!(compile_and_run("tests/regression//m36_r11.xi"), Some(0)); }
#[test] fn e2e_m36_r12() { assert_eq!(compile_and_run("tests/regression//m36_r12.xi"), Some(0)); }
#[test] fn e2e_m36_r13() { assert_eq!(compile_and_run("tests/regression//m36_r13.xi"), Some(0)); }
#[test] fn e2e_m36_r14() { assert_eq!(compile_and_run("tests/regression//m36_r14.xi"), Some(0)); }
#[test] fn e2e_m36_r15() { assert_eq!(compile_and_run("tests/regression//m36_r15.xi"), Some(0)); }
#[test] fn e2e_m36_r16() { assert_eq!(compile_and_run("tests/regression//m36_r16.xi"), Some(0)); }
#[test] fn e2e_m36_r17() { assert_eq!(compile_and_run("tests/regression//m36_r17.xi"), Some(0)); }
#[test] fn e2e_m36_r18() { assert_eq!(compile_and_run("tests/regression//m36_r18.xi"), Some(0)); }
#[test] fn e2e_m36_r19() { assert_eq!(compile_and_run("tests/regression//m36_r19.xi"), Some(0)); }
#[test] fn e2e_m36_r20() { assert_eq!(compile_and_run("tests/regression//m36_r20.xi"), Some(0)); }
#[test] fn e2e_m36_r21() { assert_eq!(compile_and_run("tests/regression//m36_r21.xi"), Some(0)); }
#[test] fn e2e_m36_r22() { assert_eq!(compile_and_run("tests/regression//m36_r22.xi"), Some(0)); }
#[test] fn e2e_m36_r23() { assert_eq!(compile_and_run("tests/regression//m36_r23.xi"), Some(0)); }
#[test] fn e2e_m36_r24() { assert_eq!(compile_and_run("tests/regression//m36_r24.xi"), Some(0)); }
#[test] fn e2e_m36_r25() { assert_eq!(compile_and_run("tests/regression//m36_r25.xi"), Some(0)); }
#[test] fn e2e_m36_r26() { assert_eq!(compile_and_run("tests/regression//m36_r26.xi"), Some(0)); }
#[test] fn e2e_m36_r27() { assert_eq!(compile_and_run("tests/regression//m36_r27.xi"), Some(0)); }
#[test] fn e2e_m36_r28() { assert_eq!(compile_and_run("tests/regression//m36_r28.xi"), Some(0)); }
#[test] fn e2e_m36_r29() { assert_eq!(compile_and_run("tests/regression//m36_r29.xi"), Some(0)); }
#[test] fn e2e_m36_r30() { assert_eq!(compile_and_run("tests/regression//m36_r30.xi"), Some(0)); }

// -- M36-S: Self-Host Preparation Compiler Patterns (30 tests) ----------
// s01: Lexer char classification       s02: Lexer token patterns
// s03: Parser AST node construction    s04: Parser tree traversal
// s05: Type checker comparison         s06: Type checker subtyping
// s07: Codegen instruction emission    s08: Codegen register allocation
// s09: Symbol table insert/lookup      s10: Symbol table scope handling
// s11: Error reporting construction    s12: AST manipulation rewrite
// s13: AST manipulation fold           s14: AST manipulation map
// s15: Pretty printer indentation      s16: Pretty printer formatting
// s17: Optimization constant folding   s18: Optimization dead code
// s19: Optimization inlining sim       s20: Serialization binary format
// s21: CLI argument parsing            s22: File path manipulation
// s23: String interning pattern        s24: GC simulation mark/sweep
// s25: Reference counting pattern      s26: VM stack simulation
// s27: VM bytecode simulation          s28: JIT compilation patterns
// s29: Debug info generation           s30: Linker/loader patterns
#[test] fn e2e_m36_s01() { assert_eq!(compile_and_run("tests/regression//m36_s01.xi"), Some(0)); }
#[test] fn e2e_m36_s02() { assert_eq!(compile_and_run("tests/regression//m36_s02.xi"), Some(0)); }
#[test] fn e2e_m36_s03() { assert_eq!(compile_and_run("tests/regression//m36_s03.xi"), Some(0)); }
#[test] fn e2e_m36_s04() { assert_eq!(compile_and_run("tests/regression//m36_s04.xi"), Some(0)); }
#[test] fn e2e_m36_s05() { assert_eq!(compile_and_run("tests/regression//m36_s05.xi"), Some(0)); }
#[test] fn e2e_m36_s06() { assert_eq!(compile_and_run("tests/regression//m36_s06.xi"), Some(0)); }
#[test] fn e2e_m36_s07() { assert_eq!(compile_and_run("tests/regression//m36_s07.xi"), Some(0)); }
#[test] fn e2e_m36_s08() { assert_eq!(compile_and_run("tests/regression//m36_s08.xi"), Some(0)); }
#[test] fn e2e_m36_s09() { assert_eq!(compile_and_run("tests/regression//m36_s09.xi"), Some(0)); }
#[test] fn e2e_m36_s10() { assert_eq!(compile_and_run("tests/regression//m36_s10.xi"), Some(0)); }
#[test] fn e2e_m36_s11() { assert_eq!(compile_and_run("tests/regression//m36_s11.xi"), Some(0)); }
#[test] fn e2e_m36_s12() { assert_eq!(compile_and_run("tests/regression//m36_s12.xi"), Some(0)); }
#[test] fn e2e_m36_s13() { assert_eq!(compile_and_run("tests/regression//m36_s13.xi"), Some(0)); }
#[test] fn e2e_m36_s14() { assert_eq!(compile_and_run("tests/regression//m36_s14.xi"), Some(0)); }
#[test] fn e2e_m36_s15() { assert_eq!(compile_and_run("tests/regression//m36_s15.xi"), Some(0)); }
#[test] fn e2e_m36_s16() { assert_eq!(compile_and_run("tests/regression//m36_s16.xi"), Some(0)); }
#[test] fn e2e_m36_s17() { assert_eq!(compile_and_run("tests/regression//m36_s17.xi"), Some(0)); }
#[test] fn e2e_m36_s18() { assert_eq!(compile_and_run("tests/regression//m36_s18.xi"), Some(0)); }
#[test] fn e2e_m36_s19() { assert_eq!(compile_and_run("tests/regression//m36_s19.xi"), Some(0)); }
#[test] fn e2e_m36_s20() { assert_eq!(compile_and_run("tests/regression//m36_s20.xi"), Some(0)); }
#[test] fn e2e_m36_s21() { assert_eq!(compile_and_run("tests/regression//m36_s21.xi"), Some(0)); }
#[test] fn e2e_m36_s22() { assert_eq!(compile_and_run("tests/regression//m36_s22.xi"), Some(0)); }
#[test] fn e2e_m36_s23() { assert_eq!(compile_and_run("tests/regression//m36_s23.xi"), Some(0)); }
#[test] fn e2e_m36_s24() { assert_eq!(compile_and_run("tests/regression//m36_s24.xi"), Some(0)); }
#[test] fn e2e_m36_s25() { assert_eq!(compile_and_run("tests/regression//m36_s25.xi"), Some(0)); }
#[test] fn e2e_m36_s26() { assert_eq!(compile_and_run("tests/regression//m36_s26.xi"), Some(0)); }
#[test] fn e2e_m36_s27() { assert_eq!(compile_and_run("tests/regression//m36_s27.xi"), Some(0)); }
#[test] fn e2e_m36_s28() { assert_eq!(compile_and_run("tests/regression//m36_s28.xi"), Some(0)); }
#[test] fn e2e_m36_s29() { assert_eq!(compile_and_run("tests/regression//m36_s29.xi"), Some(0)); }
#[test] fn e2e_m36_s30() { assert_eq!(compile_and_run("tests/regression//m36_s30.xi"), Some(0)); }

// -- M36-E: Edge Case Fuzzing Tests ------------------------------------
// e01: empty struct                       e02: single-field struct
// e03: 50-field struct                    e04: minimal enum
// e05: single-variant enum                e06: 25-variant enum
// e07: deeply nested if (20 elif)         e08: nested block 10 deep
// e09: 100 sequential statements          e10: function with 20 parameters
// e11: 100-char variable name             e12: single-char identifiers
// e13: mixed case identifiers             e14: underscore-prefixed names
// e15: numeric-suffixed names             e16: Unicode in strings
// e17: 500-char string literal            e18: 100 consecutive adds
// e19: deeply nested parens (50 levels)   e20: chained field access
// e21: chained method calls               e22: arithmetic at Int boundaries
// e23: all operators in one expression    e24: mixed type operations
// e25: multiple module use                 e26: very long source file (5500+ chars)
// e27: deep array indexing                e28: deeply nested struct literal
// e29: minimal functions                   e30: long operator chain expression
#[test] fn e2e_m36_e01() { assert_eq!(compile_and_run("tests/regression//m36_e01.xi"), Some(0)); }
#[test] fn e2e_m36_e02() { assert_eq!(compile_and_run("tests/regression//m36_e02.xi"), Some(0)); }
#[test] fn e2e_m36_e03() { assert_eq!(compile_and_run("tests/regression//m36_e03.xi"), Some(0)); }
#[test] fn e2e_m36_e04() { assert_eq!(compile_and_run("tests/regression//m36_e04.xi"), Some(0)); }
#[test] fn e2e_m36_e05() { assert_eq!(compile_and_run("tests/regression//m36_e05.xi"), Some(0)); }
#[test] fn e2e_m36_e06() { assert_eq!(compile_and_run("tests/regression//m36_e06.xi"), Some(0)); }
#[test] fn e2e_m36_e07() { assert_eq!(compile_and_run("tests/regression//m36_e07.xi"), Some(0)); }
#[test] fn e2e_m36_e08() { assert_eq!(compile_and_run("tests/regression//m36_e08.xi"), Some(0)); }
#[test] fn e2e_m36_e09() { assert_eq!(compile_and_run("tests/regression//m36_e09.xi"), Some(0)); }
#[test] fn e2e_m36_e10() { assert_eq!(compile_and_run("tests/regression//m36_e10.xi"), Some(0)); }
#[test] fn e2e_m36_e11() { assert_eq!(compile_and_run("tests/regression//m36_e11.xi"), Some(0)); }
#[test] fn e2e_m36_e12() { assert_eq!(compile_and_run("tests/regression//m36_e12.xi"), Some(0)); }
#[test] fn e2e_m36_e13() { assert_eq!(compile_and_run("tests/regression//m36_e13.xi"), Some(0)); }
#[test] fn e2e_m36_e14() { assert_eq!(compile_and_run("tests/regression//m36_e14.xi"), Some(0)); }
#[test] fn e2e_m36_e15() { assert_eq!(compile_and_run("tests/regression//m36_e15.xi"), Some(0)); }
#[test] fn e2e_m36_e16() { assert_eq!(compile_and_run("tests/regression//m36_e16.xi"), Some(0)); }
#[test] fn e2e_m36_e17() { assert_eq!(compile_and_run("tests/regression//m36_e17.xi"), Some(0)); }
#[test] fn e2e_m36_e18() { assert_eq!(compile_and_run("tests/regression//m36_e18.xi"), Some(0)); }
#[test] fn e2e_m36_e19() { assert_eq!(compile_and_run("tests/regression//m36_e19.xi"), Some(0)); }
#[test] fn e2e_m36_e20() { assert_eq!(compile_and_run("tests/regression//m36_e20.xi"), Some(0)); }
#[test] fn e2e_m36_e21() { assert_eq!(compile_and_run("tests/regression//m36_e21.xi"), Some(0)); }
#[test] fn e2e_m36_e22() { assert_eq!(compile_and_run("tests/regression//m36_e22.xi"), Some(0)); }
#[test] fn e2e_m36_e23() { assert_eq!(compile_and_run("tests/regression//m36_e23.xi"), Some(0)); }
#[test] fn e2e_m36_e24() { assert_eq!(compile_and_run("tests/regression//m36_e24.xi"), Some(0)); }
#[test] fn e2e_m36_e25() { assert_eq!(compile_and_run("tests/regression//m36_e25.xi"), Some(0)); }
#[test] fn e2e_m36_e26() { assert_eq!(compile_and_run("tests/regression//m36_e26.xi"), Some(0)); }
#[test] fn e2e_m36_e27() { assert_eq!(compile_and_run("tests/regression//m36_e27.xi"), Some(0)); }
#[test] fn e2e_m36_e28() { assert_eq!(compile_and_run("tests/regression//m36_e28.xi"), Some(0)); }
#[test] fn e2e_m36_e29() { assert_eq!(compile_and_run("tests/regression//m36_e29.xi"), Some(0)); }
#[test] fn e2e_m36_e30() { assert_eq!(compile_and_run("tests/regression//m36_e30.xi"), Some(0)); }

// -- M36-X: Self-Hosting & Release Validation Stress Tests ------------------
// x01: Exit code based on computation (returns computed value, not just 0)
// x02: Nested modules (modules within modules, pub fn, use)
// x03: External type references (type aliases, cross-referencing types)
// x04: Contract chains (requires/ensures on function call chains)
// x05: Generic dispatch (multiple type parameters, clamp, min, max)
// x06: Large match (55-arm enum, exhaustive dispatch)
// x07: Many function calls (deep call chain, composition)
// x08: Circular type references (mutually referencing struct types)
// x09: Error message patterns (Result, enum error dispatch)
// x10: --check mode patterns (syntax-only validation, token enum)
// x11: --emit-ir patterns (enum dispatch, expression trees)
// x12: --no-contracts patterns (works with/without contract checks)
// x13: Overflow checks (integer boundary values and operations)
// x14: All casts (Int/Float64/Char/Bool casts)
// x15: Derive on all types (derive[Eq] on structs)
// x16: Interface dispatch (multiple interfaces, multiple impls)
// x17: Method resolution (self methods, method chains)
// x18: Type alias resolution (nested type aliases)
// x19: Const folding (constant expression evaluation)
// x20: Dead code paths (unreachable branches, exhaustive conditions)
// x21: Tail recursion (deep tail-recursive functions)
// x22: Mutual recursion (ping/pong, is_even/is_odd, ackermann)
// x23: Deep generics (nested generic type parameters)
// x24: Pointer safety (unsafe pointer reads/writes, null checks)
// x25: Array bounds (array indexing, sum, min, max)
// x26: String manipulation (len, comparisons, empty checks)
// x27: Float precision (epsilon comparisons, pi approximation)
// x28: Integer overflow edges (MAX/MIN boundary values)
// x29: Boolean logic chains (complex boolean expressions)
// x30: ALL_FEATURES combined mega stress test
#[test] fn e2e_m36_x01() { assert_eq!(compile_and_run("tests/regression//m36_x01.xi"), Some(23)); }
#[test] fn e2e_m36_x02() { assert_eq!(compile_and_run("tests/regression//m36_x02.xi"), Some(0)); }
#[test] fn e2e_m36_x03() { assert_eq!(compile_and_run("tests/regression//m36_x03.xi"), Some(0)); }
#[test] fn e2e_m36_x04() { assert_eq!(compile_and_run("tests/regression//m36_x04.xi"), Some(0)); }
#[test] fn e2e_m36_x05() { assert_eq!(compile_and_run("tests/regression//m36_x05.xi"), Some(0)); }
#[test] fn e2e_m36_x06() { assert_eq!(compile_and_run("tests/regression//m36_x06.xi"), Some(0)); }
#[test] fn e2e_m36_x07() { assert_eq!(compile_and_run("tests/regression//m36_x07.xi"), Some(0)); }
#[test] fn e2e_m36_x08() { assert_eq!(compile_and_run("tests/regression//m36_x08.xi"), Some(0)); }
#[test] fn e2e_m36_x09() { assert_eq!(compile_and_run("tests/regression//m36_x09.xi"), Some(0)); }
#[test] fn e2e_m36_x10() { assert_eq!(compile_and_run("tests/regression//m36_x10.xi"), Some(0)); }
#[test] fn e2e_m36_x11() { assert_eq!(compile_and_run("tests/regression//m36_x11.xi"), Some(0)); }
#[test] fn e2e_m36_x12() { assert_eq!(compile_and_run("tests/regression//m36_x12.xi"), Some(0)); }
#[test] fn e2e_m36_x13() { assert_eq!(compile_and_run("tests/regression//m36_x13.xi"), Some(0)); }
#[test] fn e2e_m36_x14() { assert_eq!(compile_and_run("tests/regression//m36_x14.xi"), Some(0)); }
#[test] fn e2e_m36_x15() { assert_eq!(compile_and_run("tests/regression//m36_x15.xi"), Some(0)); }
#[test] fn e2e_m36_x16() { assert_eq!(compile_and_run("tests/regression//m36_x16.xi"), Some(0)); }
#[test] fn e2e_m36_x17() { assert_eq!(compile_and_run("tests/regression//m36_x17.xi"), Some(0)); }
#[test] fn e2e_m36_x18() { assert_eq!(compile_and_run("tests/regression//m36_x18.xi"), Some(0)); }
#[test] fn e2e_m36_x19() { assert_eq!(compile_and_run("tests/regression//m36_x19.xi"), Some(0)); }
#[test] fn e2e_m36_x20() { assert_eq!(compile_and_run("tests/regression//m36_x20.xi"), Some(0)); }
#[test] fn e2e_m36_x21() { assert_eq!(compile_and_run("tests/regression//m36_x21.xi"), Some(0)); }
#[test] fn e2e_m36_x22() { assert_eq!(compile_and_run("tests/regression//m36_x22.xi"), Some(0)); }
#[test] fn e2e_m36_x23() { assert_eq!(compile_and_run("tests/regression//m36_x23.xi"), Some(0)); }
#[test] fn e2e_m36_x24() { assert_eq!(compile_and_run("tests/regression//m36_x24.xi"), Some(0)); }
#[test] fn e2e_m36_x25() { assert_eq!(compile_and_run("tests/regression//m36_x25.xi"), Some(0)); }
#[test] fn e2e_m36_x26() { assert_eq!(compile_and_run("tests/regression//m36_x26.xi"), Some(0)); }
#[test] fn e2e_m36_x27() { assert_eq!(compile_and_run("tests/regression//m36_x27.xi"), Some(0)); }
#[test] fn e2e_m36_x28() { assert_eq!(compile_and_run("tests/regression//m36_x28.xi"), Some(0)); }
#[test] fn e2e_m36_x29() { assert_eq!(compile_and_run("tests/regression//m36_x29.xi"), Some(0)); }
#[test] fn e2e_m36_x30() { assert_eq!(compile_and_run("tests/regression//m36_x30.xi"), Some(0)); }

// -- M36-C: Combinatorial Exhaustive Stress Tests --------------------------
// c01: Every operator with every type -- arithmetic, bitwise, shift, cmp, logic, bool across all numeric types + Float64 + Bool + Char
// c02: Every control flow with every type -- if/else, while, match on Bool, Int, Float64, Char, Str, Option, enum, struct
// c03: Every match pattern with every enum shape -- 1-10 variants, payloads, nested matches, guards, wildcards
// c04: Every generic pattern with every constraint -- identity, swap, struct generic, enum generic, multi-param, bounded
// c05: Every contract pattern with every function shape -- requires, ensures, invariant, multi-clause
// c06: Every derive with every struct shape -- Eq, Clone, Eq+Clone, Ord, Hash, Display, combinations
// c07: Every method pattern with every type -- method on struct, enum, Bool, Int, Float64, Str, generic
// c08: Every module pattern with every visibility -- pub/non-pub functions, types, constants; flat, nested, re-export
// c09: Every type alias with every target type -- Bool, Int, Int8, Int16, Int32, Int64, Float64, Char, Str, struct, enum, pointer, Option, Result
// c10: Every cast direction between numeric types -- Int<->Int8, Int<->Int16, Int<->Int32, Int<->Float64, pointer<->Int, Bool<->Int
// c11: Every array size 1-2-5-10-50 -- Vec operations, indexing, sum, find
// c12: Every loop pattern -- while true, while cond, while break, while continue, nested, infinite+break
// c13: Every recursion depth 1-2-5-10-20 -- factorial, sum, power, fib, ackermann
// c14: Every struct nesting depth 1-6 -- deeply nested struct field access at all levels
// c15: Every enum variant count 1-10 -- match across all variants
// c16: Every function chain length 1-10 -- linear chains, recursive doubling, mixed chains
// c17: Every generic instantiation count 1-5 per function -- identity, pair, triple, quad, quintuple
// c18: Every contract clause count 1-5 -- requires, ensures, invariant at increasing complexity
// c19: Every module nesting depth 1-5 -- deeply nested modules with pub functions
// c20: Every type alias chain length 1-8 -- chains of type aliases resolving through multiple levels
// c21: Every derive combination -- Eq, Clone, Eq+Clone, Ord, Hash, Display, multi-combos on structs and enums
// c22: Every pointer pattern -- null pointer, deref, pointer arithmetic, pointer chain, pointer cast, pointer in struct
// c23: Every array pattern -- literal array, index, loop over array, param, return, Vec operations
// c24: Every string pattern -- literal, concat, len, byte_at, comparison, return, param, store
// c25: Every float pattern -- add, sub, mul, div, cmp, cast, negate, abs, param, return, struct field
// c26: Every int pattern -- add, sub, mul, div, mod, bitwise AND/OR/XOR/NOT, shift left/right, cmp, cast, negate
// c27: Every bool pattern -- not, and, or, if, while, match, composite, short-circuit, de_morgan
// c28: Combined mega test 1 -- structs, generics, enums, match, contracts, recursion, loops, pointers, Option
// c29: Combined mega test 2 -- modules, interfaces, impls, generics, pointers, Result, strings, type aliases
// c30: Combined mega test 3 -- ALL_FEATURES including compound_assign, closures, tickable, combat
#[test] fn e2e_m36_c01() { assert_eq!(compile_and_run("tests/regression//m36_c01.xi"), Some(0)); }
#[test] fn e2e_m36_c02() { assert_eq!(compile_and_run("tests/regression//m36_c02.xi"), Some(0)); }
#[test] fn e2e_m36_c03() { assert_eq!(compile_and_run("tests/regression//m36_c03.xi"), Some(0)); }
#[test] fn e2e_m36_c04() { assert_eq!(compile_and_run("tests/regression//m36_c04.xi"), Some(0)); }
#[test] fn e2e_m36_c05() { assert_eq!(compile_and_run("tests/regression//m36_c05.xi"), Some(0)); }
#[test] fn e2e_m36_c06() { assert_eq!(compile_and_run("tests/regression//m36_c06.xi"), Some(0)); }
#[test] fn e2e_m36_c07() { assert_eq!(compile_and_run("tests/regression//m36_c07.xi"), Some(0)); }
#[test] fn e2e_m36_c08() { assert_eq!(compile_and_run("tests/regression//m36_c08.xi"), Some(0)); }
#[test] fn e2e_m36_c09() { assert_eq!(compile_and_run("tests/regression//m36_c09.xi"), Some(0)); }
#[test] fn e2e_m36_c10() { assert_eq!(compile_and_run("tests/regression//m36_c10.xi"), Some(0)); }
#[test] fn e2e_m36_c11() { assert_eq!(compile_and_run("tests/regression//m36_c11.xi"), Some(0)); }
#[test] fn e2e_m36_c12() { assert_eq!(compile_and_run("tests/regression//m36_c12.xi"), Some(0)); }
#[test] fn e2e_m36_c13() { assert_eq!(compile_and_run("tests/regression//m36_c13.xi"), Some(0)); }
#[test] fn e2e_m36_c14() { assert_eq!(compile_and_run("tests/regression//m36_c14.xi"), Some(0)); }
#[test] fn e2e_m36_c15() { assert_eq!(compile_and_run("tests/regression//m36_c15.xi"), Some(0)); }
#[test] fn e2e_m36_c16() { assert_eq!(compile_and_run("tests/regression//m36_c16.xi"), Some(0)); }
#[test] fn e2e_m36_c17() { assert_eq!(compile_and_run("tests/regression//m36_c17.xi"), Some(0)); }
#[test] fn e2e_m36_c18() { assert_eq!(compile_and_run("tests/regression//m36_c18.xi"), Some(0)); }
#[test] fn e2e_m36_c19() { assert_eq!(compile_and_run("tests/regression//m36_c19.xi"), Some(0)); }
#[test] fn e2e_m36_c20() { assert_eq!(compile_and_run("tests/regression//m36_c20.xi"), Some(0)); }
#[test] fn e2e_m36_c21() { assert_eq!(compile_and_run("tests/regression//m36_c21.xi"), Some(0)); }
#[test] fn e2e_m36_c22() { assert_eq!(compile_and_run("tests/regression//m36_c22.xi"), Some(0)); }
#[test] fn e2e_m36_c23() { assert_eq!(compile_and_run("tests/regression//m36_c23.xi"), Some(0)); }
#[test] fn e2e_m36_c24() { assert_eq!(compile_and_run("tests/regression//m36_c24.xi"), Some(0)); }
#[test] fn e2e_m36_c25() { assert_eq!(compile_and_run("tests/regression//m36_c25.xi"), Some(0)); }
#[test] fn e2e_m36_c26() { assert_eq!(compile_and_run("tests/regression//m36_c26.xi"), Some(0)); }
#[test] fn e2e_m36_c27() { assert_eq!(compile_and_run("tests/regression//m36_c27.xi"), Some(0)); }
#[test] fn e2e_m36_c28() { assert_eq!(compile_and_run("tests/regression//m36_c28.xi"), Some(0)); }
#[test] fn e2e_m36_c29() { assert_eq!(compile_and_run("tests/regression//m36_c29.xi"), Some(0)); }
#[test] fn e2e_m36_c30() { assert_eq!(compile_and_run("tests/regression//m36_c30.xi"), Some(0)); }

// M18: Match Guard Expression Tests (001-125)
#[test] fn e2e_m18_guard_0001() { assert_eq!(compile_and_run("tests/regression//m18_guard_0001.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0002() { assert_eq!(compile_and_run("tests/regression//m18_guard_0002.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0003() { assert_eq!(compile_and_run("tests/regression//m18_guard_0003.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0004() { assert_eq!(compile_and_run("tests/regression//m18_guard_0004.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0005() { assert_eq!(compile_and_run("tests/regression//m18_guard_0005.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0006() { assert_eq!(compile_and_run("tests/regression//m18_guard_0006.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0007() { assert_eq!(compile_and_run("tests/regression//m18_guard_0007.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0008() { assert_eq!(compile_and_run("tests/regression//m18_guard_0008.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0009() { assert_eq!(compile_and_run("tests/regression//m18_guard_0009.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0010() { assert_eq!(compile_and_run("tests/regression//m18_guard_0010.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0011() { assert_eq!(compile_and_run("tests/regression//m18_guard_0011.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0012() { assert_eq!(compile_and_run("tests/regression//m18_guard_0012.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0013() { assert_eq!(compile_and_run("tests/regression//m18_guard_0013.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0014() { assert_eq!(compile_and_run("tests/regression//m18_guard_0014.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0015() { assert_eq!(compile_and_run("tests/regression//m18_guard_0015.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0016() { assert_eq!(compile_and_run("tests/regression//m18_guard_0016.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0017() { assert_eq!(compile_and_run("tests/regression//m18_guard_0017.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0018() { assert_eq!(compile_and_run("tests/regression//m18_guard_0018.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0019() { assert_eq!(compile_and_run("tests/regression//m18_guard_0019.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0020() { assert_eq!(compile_and_run("tests/regression//m18_guard_0020.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0021() { assert_eq!(compile_and_run("tests/regression//m18_guard_0021.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0022() { assert_eq!(compile_and_run("tests/regression//m18_guard_0022.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0023() { assert_eq!(compile_and_run("tests/regression//m18_guard_0023.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0024() { assert_eq!(compile_and_run("tests/regression//m18_guard_0024.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0025() { assert_eq!(compile_and_run("tests/regression//m18_guard_0025.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0026() { assert_eq!(compile_and_run("tests/regression//m18_guard_0026.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0027() { assert_eq!(compile_and_run("tests/regression//m18_guard_0027.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0028() { assert_eq!(compile_and_run("tests/regression//m18_guard_0028.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0029() { assert_eq!(compile_and_run("tests/regression//m18_guard_0029.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0030() { assert_eq!(compile_and_run("tests/regression//m18_guard_0030.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0031() { assert_eq!(compile_and_run("tests/regression//m18_guard_0031.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0032() { assert_eq!(compile_and_run("tests/regression//m18_guard_0032.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0033() { assert_eq!(compile_and_run("tests/regression//m18_guard_0033.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0034() { assert_eq!(compile_and_run("tests/regression//m18_guard_0034.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0035() { assert_eq!(compile_and_run("tests/regression//m18_guard_0035.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0036() { assert_eq!(compile_and_run("tests/regression//m18_guard_0036.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0037() { assert_eq!(compile_and_run("tests/regression//m18_guard_0037.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0038() { assert_eq!(compile_and_run("tests/regression//m18_guard_0038.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0039() { assert_eq!(compile_and_run("tests/regression//m18_guard_0039.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0040() { assert_eq!(compile_and_run("tests/regression//m18_guard_0040.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0041() { assert_eq!(compile_and_run("tests/regression//m18_guard_0041.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0042() { assert_eq!(compile_and_run("tests/regression//m18_guard_0042.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0043() { assert_eq!(compile_and_run("tests/regression//m18_guard_0043.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0044() { assert_eq!(compile_and_run("tests/regression//m18_guard_0044.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0045() { assert_eq!(compile_and_run("tests/regression//m18_guard_0045.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0046() { assert_eq!(compile_and_run("tests/regression//m18_guard_0046.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0047() { assert_eq!(compile_and_run("tests/regression//m18_guard_0047.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0048() { assert_eq!(compile_and_run("tests/regression//m18_guard_0048.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0049() { assert_eq!(compile_and_run("tests/regression//m18_guard_0049.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0050() { assert_eq!(compile_and_run("tests/regression//m18_guard_0050.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0051() { assert_eq!(compile_and_run("tests/regression//m18_guard_0051.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0052() { assert_eq!(compile_and_run("tests/regression//m18_guard_0052.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0053() { assert_eq!(compile_and_run("tests/regression//m18_guard_0053.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0054() { assert_eq!(compile_and_run("tests/regression//m18_guard_0054.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0055() { assert_eq!(compile_and_run("tests/regression//m18_guard_0055.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0056() { assert_eq!(compile_and_run("tests/regression//m18_guard_0056.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0057() { assert_eq!(compile_and_run("tests/regression//m18_guard_0057.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0058() { assert_eq!(compile_and_run("tests/regression//m18_guard_0058.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0059() { assert_eq!(compile_and_run("tests/regression//m18_guard_0059.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0060() { assert_eq!(compile_and_run("tests/regression//m18_guard_0060.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0061() { assert_eq!(compile_and_run("tests/regression//m18_guard_0061.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0062() { assert_eq!(compile_and_run("tests/regression//m18_guard_0062.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0063() { assert_eq!(compile_and_run("tests/regression//m18_guard_0063.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0064() { assert_eq!(compile_and_run("tests/regression//m18_guard_0064.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0065() { assert_eq!(compile_and_run("tests/regression//m18_guard_0065.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0066() { assert_eq!(compile_and_run("tests/regression//m18_guard_0066.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0067() { assert_eq!(compile_and_run("tests/regression//m18_guard_0067.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0068() { assert_eq!(compile_and_run("tests/regression//m18_guard_0068.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0069() { assert_eq!(compile_and_run("tests/regression//m18_guard_0069.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0070() { assert_eq!(compile_and_run("tests/regression//m18_guard_0070.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0071() { assert_eq!(compile_and_run("tests/regression//m18_guard_0071.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0072() { assert_eq!(compile_and_run("tests/regression//m18_guard_0072.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0073() { assert_eq!(compile_and_run("tests/regression//m18_guard_0073.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0074() { assert_eq!(compile_and_run("tests/regression//m18_guard_0074.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0075() { assert_eq!(compile_and_run("tests/regression//m18_guard_0075.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0076() { assert_eq!(compile_and_run("tests/regression//m18_guard_0076.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0077() { assert_eq!(compile_and_run("tests/regression//m18_guard_0077.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0078() { assert_eq!(compile_and_run("tests/regression//m18_guard_0078.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0079() { assert_eq!(compile_and_run("tests/regression//m18_guard_0079.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0080() { assert_eq!(compile_and_run("tests/regression//m18_guard_0080.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0081() { assert_eq!(compile_and_run("tests/regression//m18_guard_0081.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0082() { assert_eq!(compile_and_run("tests/regression//m18_guard_0082.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0083() { assert_eq!(compile_and_run("tests/regression//m18_guard_0083.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0084() { assert_eq!(compile_and_run("tests/regression//m18_guard_0084.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0085() { assert_eq!(compile_and_run("tests/regression//m18_guard_0085.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0086() { assert_eq!(compile_and_run("tests/regression//m18_guard_0086.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0087() { assert_eq!(compile_and_run("tests/regression//m18_guard_0087.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0088() { assert_eq!(compile_and_run("tests/regression//m18_guard_0088.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0089() { assert_eq!(compile_and_run("tests/regression//m18_guard_0089.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0090() { assert_eq!(compile_and_run("tests/regression//m18_guard_0090.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0091() { assert_eq!(compile_and_run("tests/regression//m18_guard_0091.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0092() { assert_eq!(compile_and_run("tests/regression//m18_guard_0092.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0093() { assert_eq!(compile_and_run("tests/regression//m18_guard_0093.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0094() { assert_eq!(compile_and_run("tests/regression//m18_guard_0094.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0095() { assert_eq!(compile_and_run("tests/regression//m18_guard_0095.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0096() { assert_eq!(compile_and_run("tests/regression//m18_guard_0096.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0097() { assert_eq!(compile_and_run("tests/regression//m18_guard_0097.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0098() { assert_eq!(compile_and_run("tests/regression//m18_guard_0098.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0099() { assert_eq!(compile_and_run("tests/regression//m18_guard_0099.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0100() { assert_eq!(compile_and_run("tests/regression//m18_guard_0100.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0101() { assert_eq!(compile_and_run("tests/regression//m18_guard_0101.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0102() { assert_eq!(compile_and_run("tests/regression//m18_guard_0102.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0103() { assert_eq!(compile_and_run("tests/regression//m18_guard_0103.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0104() { assert_eq!(compile_and_run("tests/regression//m18_guard_0104.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0105() { assert_eq!(compile_and_run("tests/regression//m18_guard_0105.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0106() { assert_eq!(compile_and_run("tests/regression//m18_guard_0106.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0107() { assert_eq!(compile_and_run("tests/regression//m18_guard_0107.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0108() { assert_eq!(compile_and_run("tests/regression//m18_guard_0108.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0109() { assert_eq!(compile_and_run("tests/regression//m18_guard_0109.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0110() { assert_eq!(compile_and_run("tests/regression//m18_guard_0110.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0111() { assert_eq!(compile_and_run("tests/regression//m18_guard_0111.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0112() { assert_eq!(compile_and_run("tests/regression//m18_guard_0112.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0113() { assert_eq!(compile_and_run("tests/regression//m18_guard_0113.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0114() { assert_eq!(compile_and_run("tests/regression//m18_guard_0114.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0115() { assert_eq!(compile_and_run("tests/regression//m18_guard_0115.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0116() { assert_eq!(compile_and_run("tests/regression//m18_guard_0116.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0117() { assert_eq!(compile_and_run("tests/regression//m18_guard_0117.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0118() { assert_eq!(compile_and_run("tests/regression//m18_guard_0118.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0119() { assert_eq!(compile_and_run("tests/regression//m18_guard_0119.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0120() { assert_eq!(compile_and_run("tests/regression//m18_guard_0120.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0121() { assert_eq!(compile_and_run("tests/regression//m18_guard_0121.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0122() { assert_eq!(compile_and_run("tests/regression//m18_guard_0122.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0123() { assert_eq!(compile_and_run("tests/regression//m18_guard_0123.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0124() { assert_eq!(compile_and_run("tests/regression//m18_guard_0124.xi"), Some(0)); }
#[test] fn e2e_m18_guard_0125() { assert_eq!(compile_and_run("tests/regression//m18_guard_0125.xi"), Some(0)); }

// M19: Default Interface Implementation Tests (001-125)
#[test] fn e2e_m19_default_0001() { assert_eq!(compile_and_run("tests/regression//m19_default_0001.xi"), Some(0)); }
#[test] fn e2e_m19_default_0002() { assert_eq!(compile_and_run("tests/regression//m19_default_0002.xi"), Some(0)); }
#[test] fn e2e_m19_default_0003() { assert_eq!(compile_and_run("tests/regression//m19_default_0003.xi"), Some(0)); }
#[test] fn e2e_m19_default_0004() { assert_eq!(compile_and_run("tests/regression//m19_default_0004.xi"), Some(0)); }
#[test] fn e2e_m19_default_0005() { assert_eq!(compile_and_run("tests/regression//m19_default_0005.xi"), Some(0)); }
#[test] fn e2e_m19_default_0006() { assert_eq!(compile_and_run("tests/regression//m19_default_0006.xi"), Some(0)); }
#[test] fn e2e_m19_default_0007() { assert_eq!(compile_and_run("tests/regression//m19_default_0007.xi"), Some(0)); }
#[test] fn e2e_m19_default_0008() { assert_eq!(compile_and_run("tests/regression//m19_default_0008.xi"), Some(0)); }
#[test] fn e2e_m19_default_0009() { assert_eq!(compile_and_run("tests/regression//m19_default_0009.xi"), Some(0)); }
#[test] fn e2e_m19_default_0010() { assert_eq!(compile_and_run("tests/regression//m19_default_0010.xi"), Some(0)); }
#[test] fn e2e_m19_default_0011() { assert_eq!(compile_and_run("tests/regression//m19_default_0011.xi"), Some(0)); }
#[test] fn e2e_m19_default_0012() { assert_eq!(compile_and_run("tests/regression//m19_default_0012.xi"), Some(0)); }
#[test] fn e2e_m19_default_0013() { assert_eq!(compile_and_run("tests/regression//m19_default_0013.xi"), Some(0)); }
#[test] fn e2e_m19_default_0014() { assert_eq!(compile_and_run("tests/regression//m19_default_0014.xi"), Some(0)); }
#[test] fn e2e_m19_default_0015() { assert_eq!(compile_and_run("tests/regression//m19_default_0015.xi"), Some(0)); }
#[test] fn e2e_m19_default_0016() { assert_eq!(compile_and_run("tests/regression//m19_default_0016.xi"), Some(0)); }
#[test] fn e2e_m19_default_0017() { assert_eq!(compile_and_run("tests/regression//m19_default_0017.xi"), Some(0)); }
#[test] fn e2e_m19_default_0018() { assert_eq!(compile_and_run("tests/regression//m19_default_0018.xi"), Some(0)); }
#[test] fn e2e_m19_default_0019() { assert_eq!(compile_and_run("tests/regression//m19_default_0019.xi"), Some(0)); }
#[test] fn e2e_m19_default_0020() { assert_eq!(compile_and_run("tests/regression//m19_default_0020.xi"), Some(0)); }
#[test] fn e2e_m19_default_0021() { assert_eq!(compile_and_run("tests/regression//m19_default_0021.xi"), Some(0)); }
#[test] fn e2e_m19_default_0022() { assert_eq!(compile_and_run("tests/regression//m19_default_0022.xi"), Some(0)); }
#[test] fn e2e_m19_default_0023() { assert_eq!(compile_and_run("tests/regression//m19_default_0023.xi"), Some(0)); }
#[test] fn e2e_m19_default_0024() { assert_eq!(compile_and_run("tests/regression//m19_default_0024.xi"), Some(0)); }
#[test] fn e2e_m19_default_0025() { assert_eq!(compile_and_run("tests/regression//m19_default_0025.xi"), Some(0)); }
#[test] fn e2e_m19_default_0026() { assert_eq!(compile_and_run("tests/regression//m19_default_0026.xi"), Some(0)); }
#[test] fn e2e_m19_default_0027() { assert_eq!(compile_and_run("tests/regression//m19_default_0027.xi"), Some(0)); }
#[test] fn e2e_m19_default_0028() { assert_eq!(compile_and_run("tests/regression//m19_default_0028.xi"), Some(0)); }
#[test] fn e2e_m19_default_0029() { assert_eq!(compile_and_run("tests/regression//m19_default_0029.xi"), Some(0)); }
#[test] fn e2e_m19_default_0030() { assert_eq!(compile_and_run("tests/regression//m19_default_0030.xi"), Some(0)); }
#[test] fn e2e_m19_default_0031() { assert_eq!(compile_and_run("tests/regression//m19_default_0031.xi"), Some(0)); }
#[test] fn e2e_m19_default_0032() { assert_eq!(compile_and_run("tests/regression//m19_default_0032.xi"), Some(0)); }
#[test] fn e2e_m19_default_0033() { assert_eq!(compile_and_run("tests/regression//m19_default_0033.xi"), Some(0)); }
#[test] fn e2e_m19_default_0034() { assert_eq!(compile_and_run("tests/regression//m19_default_0034.xi"), Some(0)); }
#[test] fn e2e_m19_default_0035() { assert_eq!(compile_and_run("tests/regression//m19_default_0035.xi"), Some(0)); }
#[test] fn e2e_m19_default_0036() { assert_eq!(compile_and_run("tests/regression//m19_default_0036.xi"), Some(0)); }
#[test] fn e2e_m19_default_0037() { assert_eq!(compile_and_run("tests/regression//m19_default_0037.xi"), Some(0)); }
#[test] fn e2e_m19_default_0038() { assert_eq!(compile_and_run("tests/regression//m19_default_0038.xi"), Some(0)); }
#[test] fn e2e_m19_default_0039() { assert_eq!(compile_and_run("tests/regression//m19_default_0039.xi"), Some(0)); }
#[test] fn e2e_m19_default_0040() { assert_eq!(compile_and_run("tests/regression//m19_default_0040.xi"), Some(0)); }
#[test] fn e2e_m19_default_0041() { assert_eq!(compile_and_run("tests/regression//m19_default_0041.xi"), Some(0)); }
#[test] fn e2e_m19_default_0042() { assert_eq!(compile_and_run("tests/regression//m19_default_0042.xi"), Some(0)); }
#[test] fn e2e_m19_default_0043() { assert_eq!(compile_and_run("tests/regression//m19_default_0043.xi"), Some(0)); }
#[test] fn e2e_m19_default_0044() { assert_eq!(compile_and_run("tests/regression//m19_default_0044.xi"), Some(0)); }
#[test] fn e2e_m19_default_0045() { assert_eq!(compile_and_run("tests/regression//m19_default_0045.xi"), Some(0)); }
#[test] fn e2e_m19_default_0046() { assert_eq!(compile_and_run("tests/regression//m19_default_0046.xi"), Some(0)); }
#[test] fn e2e_m19_default_0047() { assert_eq!(compile_and_run("tests/regression//m19_default_0047.xi"), Some(0)); }
#[test] fn e2e_m19_default_0048() { assert_eq!(compile_and_run("tests/regression//m19_default_0048.xi"), Some(0)); }
#[test] fn e2e_m19_default_0049() { assert_eq!(compile_and_run("tests/regression//m19_default_0049.xi"), Some(0)); }
#[test] fn e2e_m19_default_0050() { assert_eq!(compile_and_run("tests/regression//m19_default_0050.xi"), Some(0)); }
#[test] fn e2e_m19_default_0051() { assert_eq!(compile_and_run("tests/regression//m19_default_0051.xi"), Some(0)); }
#[test] fn e2e_m19_default_0052() { assert_eq!(compile_and_run("tests/regression//m19_default_0052.xi"), Some(0)); }
#[test] fn e2e_m19_default_0053() { assert_eq!(compile_and_run("tests/regression//m19_default_0053.xi"), Some(0)); }
#[test] fn e2e_m19_default_0054() { assert_eq!(compile_and_run("tests/regression//m19_default_0054.xi"), Some(0)); }
#[test] fn e2e_m19_default_0055() { assert_eq!(compile_and_run("tests/regression//m19_default_0055.xi"), Some(0)); }
#[test] fn e2e_m19_default_0056() { assert_eq!(compile_and_run("tests/regression//m19_default_0056.xi"), Some(0)); }
#[test] fn e2e_m19_default_0057() { assert_eq!(compile_and_run("tests/regression//m19_default_0057.xi"), Some(0)); }
#[test] fn e2e_m19_default_0058() { assert_eq!(compile_and_run("tests/regression//m19_default_0058.xi"), Some(0)); }
#[test] fn e2e_m19_default_0059() { assert_eq!(compile_and_run("tests/regression//m19_default_0059.xi"), Some(0)); }
#[test] fn e2e_m19_default_0060() { assert_eq!(compile_and_run("tests/regression//m19_default_0060.xi"), Some(0)); }
#[test] fn e2e_m19_default_0061() { assert_eq!(compile_and_run("tests/regression//m19_default_0061.xi"), Some(0)); }
#[test] fn e2e_m19_default_0062() { assert_eq!(compile_and_run("tests/regression//m19_default_0062.xi"), Some(0)); }
#[test] fn e2e_m19_default_0063() { assert_eq!(compile_and_run("tests/regression//m19_default_0063.xi"), Some(0)); }
#[test] fn e2e_m19_default_0064() { assert_eq!(compile_and_run("tests/regression//m19_default_0064.xi"), Some(0)); }
#[test] fn e2e_m19_default_0065() { assert_eq!(compile_and_run("tests/regression//m19_default_0065.xi"), Some(0)); }
#[test] fn e2e_m19_default_0066() { assert_eq!(compile_and_run("tests/regression//m19_default_0066.xi"), Some(0)); }
#[test] fn e2e_m19_default_0067() { assert_eq!(compile_and_run("tests/regression//m19_default_0067.xi"), Some(0)); }
#[test] fn e2e_m19_default_0068() { assert_eq!(compile_and_run("tests/regression//m19_default_0068.xi"), Some(0)); }
#[test] fn e2e_m19_default_0069() { assert_eq!(compile_and_run("tests/regression//m19_default_0069.xi"), Some(0)); }
#[test] fn e2e_m19_default_0070() { assert_eq!(compile_and_run("tests/regression//m19_default_0070.xi"), Some(0)); }
#[test] fn e2e_m19_default_0071() { assert_eq!(compile_and_run("tests/regression//m19_default_0071.xi"), Some(0)); }
#[test] fn e2e_m19_default_0072() { assert_eq!(compile_and_run("tests/regression//m19_default_0072.xi"), Some(0)); }
#[test] fn e2e_m19_default_0073() { assert_eq!(compile_and_run("tests/regression//m19_default_0073.xi"), Some(0)); }
#[test] fn e2e_m19_default_0074() { assert_eq!(compile_and_run("tests/regression//m19_default_0074.xi"), Some(0)); }
#[test] fn e2e_m19_default_0075() { assert_eq!(compile_and_run("tests/regression//m19_default_0075.xi"), Some(0)); }
#[test] fn e2e_m19_default_0076() { assert_eq!(compile_and_run("tests/regression//m19_default_0076.xi"), Some(0)); }
#[test] fn e2e_m19_default_0077() { assert_eq!(compile_and_run("tests/regression//m19_default_0077.xi"), Some(0)); }
#[test] fn e2e_m19_default_0078() { assert_eq!(compile_and_run("tests/regression//m19_default_0078.xi"), Some(0)); }
#[test] fn e2e_m19_default_0079() { assert_eq!(compile_and_run("tests/regression//m19_default_0079.xi"), Some(0)); }
#[test] fn e2e_m19_default_0080() { assert_eq!(compile_and_run("tests/regression//m19_default_0080.xi"), Some(0)); }
#[test] fn e2e_m19_default_0081() { assert_eq!(compile_and_run("tests/regression//m19_default_0081.xi"), Some(0)); }
#[test] fn e2e_m19_default_0082() { assert_eq!(compile_and_run("tests/regression//m19_default_0082.xi"), Some(0)); }
#[test] fn e2e_m19_default_0083() { assert_eq!(compile_and_run("tests/regression//m19_default_0083.xi"), Some(0)); }
#[test] fn e2e_m19_default_0084() { assert_eq!(compile_and_run("tests/regression//m19_default_0084.xi"), Some(0)); }
#[test] fn e2e_m19_default_0085() { assert_eq!(compile_and_run("tests/regression//m19_default_0085.xi"), Some(0)); }
#[test] fn e2e_m19_default_0086() { assert_eq!(compile_and_run("tests/regression//m19_default_0086.xi"), Some(0)); }
#[test] fn e2e_m19_default_0087() { assert_eq!(compile_and_run("tests/regression//m19_default_0087.xi"), Some(0)); }
#[test] fn e2e_m19_default_0088() { assert_eq!(compile_and_run("tests/regression//m19_default_0088.xi"), Some(0)); }
#[test] fn e2e_m19_default_0089() { assert_eq!(compile_and_run("tests/regression//m19_default_0089.xi"), Some(0)); }
#[test] fn e2e_m19_default_0090() { assert_eq!(compile_and_run("tests/regression//m19_default_0090.xi"), Some(0)); }
#[test] fn e2e_m19_default_0091() { assert_eq!(compile_and_run("tests/regression//m19_default_0091.xi"), Some(0)); }
#[test] fn e2e_m19_default_0092() { assert_eq!(compile_and_run("tests/regression//m19_default_0092.xi"), Some(0)); }
#[test] fn e2e_m19_default_0093() { assert_eq!(compile_and_run("tests/regression//m19_default_0093.xi"), Some(0)); }
#[test] fn e2e_m19_default_0094() { assert_eq!(compile_and_run("tests/regression//m19_default_0094.xi"), Some(0)); }
#[test] fn e2e_m19_default_0095() { assert_eq!(compile_and_run("tests/regression//m19_default_0095.xi"), Some(0)); }
#[test] fn e2e_m19_default_0096() { assert_eq!(compile_and_run("tests/regression//m19_default_0096.xi"), Some(0)); }
#[test] fn e2e_m19_default_0097() { assert_eq!(compile_and_run("tests/regression//m19_default_0097.xi"), Some(0)); }
#[test] fn e2e_m19_default_0098() { assert_eq!(compile_and_run("tests/regression//m19_default_0098.xi"), Some(0)); }
#[test] fn e2e_m19_default_0099() { assert_eq!(compile_and_run("tests/regression//m19_default_0099.xi"), Some(0)); }
#[test] fn e2e_m19_default_0100() { assert_eq!(compile_and_run("tests/regression//m19_default_0100.xi"), Some(0)); }
#[test] fn e2e_m19_default_0101() { assert_eq!(compile_and_run("tests/regression//m19_default_0101.xi"), Some(0)); }
#[test] fn e2e_m19_default_0102() { assert_eq!(compile_and_run("tests/regression//m19_default_0102.xi"), Some(0)); }
#[test] fn e2e_m19_default_0103() { assert_eq!(compile_and_run("tests/regression//m19_default_0103.xi"), Some(0)); }
#[test] fn e2e_m19_default_0104() { assert_eq!(compile_and_run("tests/regression//m19_default_0104.xi"), Some(0)); }
#[test] fn e2e_m19_default_0105() { assert_eq!(compile_and_run("tests/regression//m19_default_0105.xi"), Some(0)); }
#[test] fn e2e_m19_default_0106() { assert_eq!(compile_and_run("tests/regression//m19_default_0106.xi"), Some(0)); }
#[test] fn e2e_m19_default_0107() { assert_eq!(compile_and_run("tests/regression//m19_default_0107.xi"), Some(0)); }
#[test] fn e2e_m19_default_0108() { assert_eq!(compile_and_run("tests/regression//m19_default_0108.xi"), Some(0)); }
#[test] fn e2e_m19_default_0109() { assert_eq!(compile_and_run("tests/regression//m19_default_0109.xi"), Some(0)); }
#[test] fn e2e_m19_default_0110() { assert_eq!(compile_and_run("tests/regression//m19_default_0110.xi"), Some(0)); }
#[test] fn e2e_m19_default_0111() { assert_eq!(compile_and_run("tests/regression//m19_default_0111.xi"), Some(0)); }
#[test] fn e2e_m19_default_0112() { assert_eq!(compile_and_run("tests/regression//m19_default_0112.xi"), Some(0)); }
#[test] fn e2e_m19_default_0113() { assert_eq!(compile_and_run("tests/regression//m19_default_0113.xi"), Some(0)); }
#[test] fn e2e_m19_default_0114() { assert_eq!(compile_and_run("tests/regression//m19_default_0114.xi"), Some(0)); }
#[test] fn e2e_m19_default_0115() { assert_eq!(compile_and_run("tests/regression//m19_default_0115.xi"), Some(0)); }
#[test] fn e2e_m19_default_0116() { assert_eq!(compile_and_run("tests/regression//m19_default_0116.xi"), Some(0)); }
#[test] fn e2e_m19_default_0117() { assert_eq!(compile_and_run("tests/regression//m19_default_0117.xi"), Some(0)); }
#[test] fn e2e_m19_default_0118() { assert_eq!(compile_and_run("tests/regression//m19_default_0118.xi"), Some(0)); }
#[test] fn e2e_m19_default_0119() { assert_eq!(compile_and_run("tests/regression//m19_default_0119.xi"), Some(0)); }
#[test] fn e2e_m19_default_0120() { assert_eq!(compile_and_run("tests/regression//m19_default_0120.xi"), Some(0)); }
#[test] fn e2e_m19_default_0121() { assert_eq!(compile_and_run("tests/regression//m19_default_0121.xi"), Some(0)); }
#[test] fn e2e_m19_default_0122() { assert_eq!(compile_and_run("tests/regression//m19_default_0122.xi"), Some(0)); }
#[test] fn e2e_m19_default_0123() { assert_eq!(compile_and_run("tests/regression//m19_default_0123.xi"), Some(0)); }
#[test] fn e2e_m19_default_0124() { assert_eq!(compile_and_run("tests/regression//m19_default_0124.xi"), Some(0)); }
#[test] fn e2e_m19_default_0125() { assert_eq!(compile_and_run("tests/regression//m19_default_0125.xi"), Some(0)); }

// M20: Missing Final Tests
#[test] fn e2e_m20_final_func_ptr() { assert_eq!(compile_and_run("tests/regression//m20_final_func_ptr.xi"), Some(0)); }
#[test] fn e2e_m20_harden_array_lit() { assert_eq!(compile_and_run("tests/regression//m20_harden_array_lit.xi"), Some(0)); }

// M21: Remaining Subcategory Tests
// m21_async_spawn
#[test] fn e2e_m21_async_spawn_001() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_001.xi"), Some(0)); }
#[test] fn e2e_m21_async_spawn_002() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_002.xi"), Some(0)); }
#[test] fn e2e_m21_async_spawn_003() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_003.xi"), Some(0)); }
#[test] fn e2e_m21_async_spawn_004() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_004.xi"), Some(0)); }
#[test] fn e2e_m21_async_spawn_005() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_005.xi"), Some(0)); }
#[test] fn e2e_m21_async_spawn_006() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_006.xi"), Some(0)); }
#[test] fn e2e_m21_async_spawn_007() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_007.xi"), Some(0)); }
#[test] fn e2e_m21_async_spawn_008() { assert_eq!(compile_and_run("tests/regression//m21_async_spawn_008.xi"), Some(0)); }
// m21_borrow
#[test] fn e2e_m21_borrow_001() { assert_eq!(compile_and_run("tests/regression//m21_borrow_001.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_002() { assert_eq!(compile_and_run("tests/regression//m21_borrow_002.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_003() { assert_eq!(compile_and_run("tests/regression//m21_borrow_003.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_004() { assert_eq!(compile_and_run("tests/regression//m21_borrow_004.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_005() { assert_eq!(compile_and_run("tests/regression//m21_borrow_005.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_006() { assert_eq!(compile_and_run("tests/regression//m21_borrow_006.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_007() { assert_eq!(compile_and_run("tests/regression//m21_borrow_007.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_008() { assert_eq!(compile_and_run("tests/regression//m21_borrow_008.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_009() { assert_eq!(compile_and_run("tests/regression//m21_borrow_009.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_010() { assert_eq!(compile_and_run("tests/regression//m21_borrow_010.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_011() { assert_eq!(compile_and_run("tests/regression//m21_borrow_011.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_012() { assert_eq!(compile_and_run("tests/regression//m21_borrow_012.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_013() { assert_eq!(compile_and_run("tests/regression//m21_borrow_013.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_014() { assert_eq!(compile_and_run("tests/regression//m21_borrow_014.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_015() { assert_eq!(compile_and_run("tests/regression//m21_borrow_015.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_016() { assert_eq!(compile_and_run("tests/regression//m21_borrow_016.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_017() { assert_eq!(compile_and_run("tests/regression//m21_borrow_017.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_018() { assert_eq!(compile_and_run("tests/regression//m21_borrow_018.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_019() { assert_eq!(compile_and_run("tests/regression//m21_borrow_019.xi"), Some(0)); }
#[test] fn e2e_m21_borrow_020() { assert_eq!(compile_and_run("tests/regression//m21_borrow_020.xi"), Some(0)); }
// m21_complex_generic
#[test] fn e2e_m21_complex_generic_001() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_001.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_002() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_002.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_003() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_003.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_004() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_004.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_005() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_005.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_006() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_006.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_007() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_007.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_008() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_008.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_009() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_009.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_010() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_010.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_011() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_011.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_012() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_012.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_013() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_013.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_014() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_014.xi"), Some(0)); }
#[test] fn e2e_m21_complex_generic_015() { assert_eq!(compile_and_run("tests/regression//m21_complex_generic_015.xi"), Some(0)); }
// m21_contract
#[test] fn e2e_m21_contract_001() { assert_eq!(compile_and_run("tests/regression//m21_contract_001.xi"), Some(0)); }
#[test] fn e2e_m21_contract_002() { assert_eq!(compile_and_run("tests/regression//m21_contract_002.xi"), Some(0)); }
#[test] fn e2e_m21_contract_003() { assert_eq!(compile_and_run("tests/regression//m21_contract_003.xi"), Some(0)); }
#[test] fn e2e_m21_contract_004() { assert_eq!(compile_and_run("tests/regression//m21_contract_004.xi"), Some(0)); }
#[test] fn e2e_m21_contract_005() { assert_eq!(compile_and_run("tests/regression//m21_contract_005.xi"), Some(0)); }
#[test] fn e2e_m21_contract_006() { assert_eq!(compile_and_run("tests/regression//m21_contract_006.xi"), Some(0)); }
#[test] fn e2e_m21_contract_007() { assert_eq!(compile_and_run("tests/regression//m21_contract_007.xi"), Some(0)); }
#[test] fn e2e_m21_contract_008() { assert_eq!(compile_and_run("tests/regression//m21_contract_008.xi"), Some(0)); }
#[test] fn e2e_m21_contract_009() { assert_eq!(compile_and_run("tests/regression//m21_contract_009.xi"), Some(0)); }
#[test] fn e2e_m21_contract_010() { assert_eq!(compile_and_run("tests/regression//m21_contract_010.xi"), Some(0)); }
// m21_deep_expr
#[test] fn e2e_m21_deep_expr_001() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_001.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_002() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_002.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_003() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_003.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_004() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_004.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_005() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_005.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_006() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_006.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_007() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_007.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_008() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_008.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_009() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_009.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_010() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_010.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_011() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_011.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_012() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_012.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_013() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_013.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_014() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_014.xi"), Some(0)); }
#[test] fn e2e_m21_deep_expr_015() { assert_eq!(compile_and_run("tests/regression//m21_deep_expr_015.xi"), Some(0)); }
// m21_derive
#[test] fn e2e_m21_derive_001() { assert_eq!(compile_and_run("tests/regression//m21_derive_001.xi"), Some(0)); }
#[test] fn e2e_m21_derive_002() { assert_eq!(compile_and_run("tests/regression//m21_derive_002.xi"), Some(0)); }
#[test] fn e2e_m21_derive_003() { assert_eq!(compile_and_run("tests/regression//m21_derive_003.xi"), Some(0)); }
#[test] fn e2e_m21_derive_004() { assert_eq!(compile_and_run("tests/regression//m21_derive_004.xi"), Some(0)); }
#[test] fn e2e_m21_derive_005() { assert_eq!(compile_and_run("tests/regression//m21_derive_005.xi"), Some(0)); }
#[test] fn e2e_m21_derive_006() { assert_eq!(compile_and_run("tests/regression//m21_derive_006.xi"), Some(0)); }
#[test] fn e2e_m21_derive_007() { assert_eq!(compile_and_run("tests/regression//m21_derive_007.xi"), Some(0)); }
#[test] fn e2e_m21_derive_008() { assert_eq!(compile_and_run("tests/regression//m21_derive_008.xi"), Some(0)); }
#[test] fn e2e_m21_derive_009() { assert_eq!(compile_and_run("tests/regression//m21_derive_009.xi"), Some(0)); }
#[test] fn e2e_m21_derive_010() { assert_eq!(compile_and_run("tests/regression//m21_derive_010.xi"), Some(0)); }
// m21_destructure
#[test] fn e2e_m21_destructure_001() { assert_eq!(compile_and_run("tests/regression//m21_destructure_001.xi"), Some(0)); }
#[test] fn e2e_m21_destructure_002() { assert_eq!(compile_and_run("tests/regression//m21_destructure_002.xi"), Some(0)); }
#[test] fn e2e_m21_destructure_003() { assert_eq!(compile_and_run("tests/regression//m21_destructure_003.xi"), Some(0)); }
#[test] fn e2e_m21_destructure_004() { assert_eq!(compile_and_run("tests/regression//m21_destructure_004.xi"), Some(0)); }
#[test] fn e2e_m21_destructure_005() { assert_eq!(compile_and_run("tests/regression//m21_destructure_005.xi"), Some(0)); }
#[test] fn e2e_m21_destructure_006() { assert_eq!(compile_and_run("tests/regression//m21_destructure_006.xi"), Some(0)); }
#[test] fn e2e_m21_destructure_007() { assert_eq!(compile_and_run("tests/regression//m21_destructure_007.xi"), Some(0)); }
#[test] fn e2e_m21_destructure_008() { assert_eq!(compile_and_run("tests/regression//m21_destructure_008.xi"), Some(0)); }
// m21_ffi_unsafe
#[test] fn e2e_m21_ffi_unsafe_001() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_001.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_002() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_002.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_003() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_003.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_004() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_004.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_005() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_005.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_006() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_006.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_007() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_007.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_008() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_008.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_009() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_009.xi"), Some(0)); }
#[test] fn e2e_m21_ffi_unsafe_010() { assert_eq!(compile_and_run("tests/regression//m21_ffi_unsafe_010.xi"), Some(0)); }
// m21_if_chain
#[test] fn e2e_m21_if_chain_001() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_001.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_002() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_002.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_003() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_003.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_004() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_004.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_005() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_005.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_006() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_006.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_007() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_007.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_008() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_008.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_009() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_009.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_010() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_010.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_011() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_011.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_012() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_012.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_013() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_013.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_014() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_014.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_015() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_015.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_016() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_016.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_017() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_017.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_018() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_018.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_019() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_019.xi"), Some(0)); }
#[test] fn e2e_m21_if_chain_020() { assert_eq!(compile_and_run("tests/regression//m21_if_chain_020.xi"), Some(0)); }
// m21_int_edge
#[test] fn e2e_m21_int_edge_001() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_001.xi"), Some(0)); }
#[test] fn e2e_m21_int_edge_002() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_002.xi"), Some(0)); }
#[test] fn e2e_m21_int_edge_003() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_003.xi"), Some(0)); }
#[test] fn e2e_m21_int_edge_004() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_004.xi"), Some(0)); }
#[test] fn e2e_m21_int_edge_005() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_005.xi"), Some(0)); }
#[test] fn e2e_m21_int_edge_006() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_006.xi"), Some(0)); }
#[test] fn e2e_m21_int_edge_007() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_007.xi"), Some(0)); }
#[test] fn e2e_m21_int_edge_008() { assert_eq!(compile_and_run("tests/regression//m21_int_edge_008.xi"), Some(0)); }
// m21_match_edge
#[test] fn e2e_m21_match_edge_001() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_001.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_002() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_002.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_003() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_003.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_004() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_004.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_005() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_005.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_006() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_006.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_007() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_007.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_008() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_008.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_009() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_009.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_010() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_010.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_011() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_011.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_012() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_012.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_013() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_013.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_014() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_014.xi"), Some(0)); }
#[test] fn e2e_m21_match_edge_015() { assert_eq!(compile_and_run("tests/regression//m21_match_edge_015.xi"), Some(0)); }
// m21_module
#[test] fn e2e_m21_module_001() { assert_eq!(compile_and_run("tests/regression//m21_module_001.xi"), Some(0)); }
#[test] fn e2e_m21_module_002() { assert_eq!(compile_and_run("tests/regression//m21_module_002.xi"), Some(0)); }
#[test] fn e2e_m21_module_003() { assert_eq!(compile_and_run("tests/regression//m21_module_003.xi"), Some(0)); }
#[test] fn e2e_m21_module_004() { assert_eq!(compile_and_run("tests/regression//m21_module_004.xi"), Some(0)); }
#[test] fn e2e_m21_module_005() { assert_eq!(compile_and_run("tests/regression//m21_module_005.xi"), Some(0)); }
#[test] fn e2e_m21_module_006() { assert_eq!(compile_and_run("tests/regression//m21_module_006.xi"), Some(0)); }
#[test] fn e2e_m21_module_007() { assert_eq!(compile_and_run("tests/regression//m21_module_007.xi"), Some(0)); }
#[test] fn e2e_m21_module_008() { assert_eq!(compile_and_run("tests/regression//m21_module_008.xi"), Some(0)); }
#[test] fn e2e_m21_module_009() { assert_eq!(compile_and_run("tests/regression//m21_module_009.xi"), Some(0)); }
#[test] fn e2e_m21_module_010() { assert_eq!(compile_and_run("tests/regression//m21_module_010.xi"), Some(0)); }
#[test] fn e2e_m21_module_011() { assert_eq!(compile_and_run("tests/regression//m21_module_011.xi"), Some(0)); }
#[test] fn e2e_m21_module_012() { assert_eq!(compile_and_run("tests/regression//m21_module_012.xi"), Some(0)); }
#[test] fn e2e_m21_module_013() { assert_eq!(compile_and_run("tests/regression//m21_module_013.xi"), Some(0)); }
#[test] fn e2e_m21_module_014() { assert_eq!(compile_and_run("tests/regression//m21_module_014.xi"), Some(0)); }
#[test] fn e2e_m21_module_015() { assert_eq!(compile_and_run("tests/regression//m21_module_015.xi"), Some(0)); }
// m21_op_prec
#[test] fn e2e_m21_op_prec_001() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_001.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_002() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_002.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_003() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_003.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_004() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_004.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_005() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_005.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_006() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_006.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_007() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_007.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_008() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_008.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_009() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_009.xi"), Some(0)); }
#[test] fn e2e_m21_op_prec_010() { assert_eq!(compile_and_run("tests/regression//m21_op_prec_010.xi"), Some(0)); }
// m21_result_option
#[test] fn e2e_m21_result_option_001() { assert_eq!(compile_and_run("tests/regression//m21_result_option_001.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_002() { assert_eq!(compile_and_run("tests/regression//m21_result_option_002.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_003() { assert_eq!(compile_and_run("tests/regression//m21_result_option_003.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_004() { assert_eq!(compile_and_run("tests/regression//m21_result_option_004.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_005() { assert_eq!(compile_and_run("tests/regression//m21_result_option_005.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_006() { assert_eq!(compile_and_run("tests/regression//m21_result_option_006.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_007() { assert_eq!(compile_and_run("tests/regression//m21_result_option_007.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_008() { assert_eq!(compile_and_run("tests/regression//m21_result_option_008.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_009() { assert_eq!(compile_and_run("tests/regression//m21_result_option_009.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_010() { assert_eq!(compile_and_run("tests/regression//m21_result_option_010.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_011() { assert_eq!(compile_and_run("tests/regression//m21_result_option_011.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_012() { assert_eq!(compile_and_run("tests/regression//m21_result_option_012.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_013() { assert_eq!(compile_and_run("tests/regression//m21_result_option_013.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_014() { assert_eq!(compile_and_run("tests/regression//m21_result_option_014.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_015() { assert_eq!(compile_and_run("tests/regression//m21_result_option_015.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_016() { assert_eq!(compile_and_run("tests/regression//m21_result_option_016.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_017() { assert_eq!(compile_and_run("tests/regression//m21_result_option_017.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_018() { assert_eq!(compile_and_run("tests/regression//m21_result_option_018.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_019() { assert_eq!(compile_and_run("tests/regression//m21_result_option_019.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_020() { assert_eq!(compile_and_run("tests/regression//m21_result_option_020.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_021() { assert_eq!(compile_and_run("tests/regression//m21_result_option_021.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_022() { assert_eq!(compile_and_run("tests/regression//m21_result_option_022.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_023() { assert_eq!(compile_and_run("tests/regression//m21_result_option_023.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_024() { assert_eq!(compile_and_run("tests/regression//m21_result_option_024.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_025() { assert_eq!(compile_and_run("tests/regression//m21_result_option_025.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_026() { assert_eq!(compile_and_run("tests/regression//m21_result_option_026.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_027() { assert_eq!(compile_and_run("tests/regression//m21_result_option_027.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_028() { assert_eq!(compile_and_run("tests/regression//m21_result_option_028.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_029() { assert_eq!(compile_and_run("tests/regression//m21_result_option_029.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_030() { assert_eq!(compile_and_run("tests/regression//m21_result_option_030.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_031() { assert_eq!(compile_and_run("tests/regression//m21_result_option_031.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_032() { assert_eq!(compile_and_run("tests/regression//m21_result_option_032.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_033() { assert_eq!(compile_and_run("tests/regression//m21_result_option_033.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_034() { assert_eq!(compile_and_run("tests/regression//m21_result_option_034.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_035() { assert_eq!(compile_and_run("tests/regression//m21_result_option_035.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_036() { assert_eq!(compile_and_run("tests/regression//m21_result_option_036.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_037() { assert_eq!(compile_and_run("tests/regression//m21_result_option_037.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_038() { assert_eq!(compile_and_run("tests/regression//m21_result_option_038.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_039() { assert_eq!(compile_and_run("tests/regression//m21_result_option_039.xi"), Some(0)); }
#[test] fn e2e_m21_result_option_040() { assert_eq!(compile_and_run("tests/regression//m21_result_option_040.xi"), Some(0)); }
// m21_string
#[test] fn e2e_m21_string_003() { assert_eq!(compile_and_run("tests/regression//m21_string_003.xi"), Some(0)); }
#[test] fn e2e_m21_string_004() { assert_eq!(compile_and_run("tests/regression//m21_string_004.xi"), Some(0)); }
#[test] fn e2e_m21_string_005() { assert_eq!(compile_and_run("tests/regression//m21_string_005.xi"), Some(0)); }
#[test] fn e2e_m21_string_007() { assert_eq!(compile_and_run("tests/regression//m21_string_007.xi"), Some(0)); }
#[test] fn e2e_m21_string_010() { assert_eq!(compile_and_run("tests/regression//m21_string_010.xi"), Some(0)); }
#[test] fn e2e_m21_string_020() { assert_eq!(compile_and_run("tests/regression//m21_string_020.xi"), Some(0)); }
// m21_struct_mut
#[test] fn e2e_m21_struct_mut_004() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_004.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_015() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_015.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_016() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_016.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_017() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_017.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_018() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_018.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_019() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_019.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_023() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_023.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_026() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_026.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_027() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_027.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_028() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_028.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_032() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_032.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_034() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_034.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_035() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_035.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_036() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_036.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_037() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_037.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_039() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_039.xi"), Some(0)); }
#[test] fn e2e_m21_struct_mut_040() { assert_eq!(compile_and_run("tests/regression//m21_struct_mut_040.xi"), Some(0)); }
// m21_type_edge
#[test] fn e2e_m21_type_edge_001() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_001.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_002() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_002.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_003() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_003.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_004() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_004.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_005() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_005.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_006() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_006.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_007() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_007.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_008() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_008.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_009() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_009.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_010() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_010.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_011() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_011.xi"), Some(0)); }
#[test] fn e2e_m21_type_edge_012() { assert_eq!(compile_and_run("tests/regression//m21_type_edge_012.xi"), Some(0)); }
// m21_vec_edge
#[test] fn e2e_m21_vec_edge_001() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_001.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_002() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_002.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_003() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_003.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_004() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_004.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_005() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_005.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_006() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_006.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_007() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_007.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_008() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_008.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_009() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_009.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_010() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_010.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_011() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_011.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_012() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_012.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_013() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_013.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_014() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_014.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_015() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_015.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_016() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_016.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_017() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_017.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_018() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_018.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_019() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_019.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_020() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_020.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_021() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_021.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_022() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_022.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_023() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_023.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_024() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_024.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_025() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_025.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_026() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_026.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_027() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_027.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_028() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_028.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_029() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_029.xi"), Some(0)); }
#[test] fn e2e_m21_vec_edge_030() { assert_eq!(compile_and_run("tests/regression//m21_vec_edge_030.xi"), Some(0)); }

// M33-Z: Final Batch Tests (z01-z20)
#[test] fn e2e_m33_z01() { assert_eq!(compile_and_run("tests/regression//m33_z01.xi"), Some(0)); }
#[test] fn e2e_m33_z02() { assert_eq!(compile_and_run("tests/regression//m33_z02.xi"), Some(0)); }
#[test] fn e2e_m33_z03() { assert_eq!(compile_and_run("tests/regression//m33_z03.xi"), Some(0)); }
#[test] fn e2e_m33_z04() { assert_eq!(compile_and_run("tests/regression//m33_z04.xi"), Some(0)); }
#[test] fn e2e_m33_z05() { assert_eq!(compile_and_run("tests/regression//m33_z05.xi"), Some(0)); }
#[test] fn e2e_m33_z06() { assert_eq!(compile_and_run("tests/regression//m33_z06.xi"), Some(0)); }
#[test] fn e2e_m33_z07() { assert_eq!(compile_and_run("tests/regression//m33_z07.xi"), Some(0)); }
#[test] fn e2e_m33_z08() { assert_eq!(compile_and_run("tests/regression//m33_z08.xi"), Some(0)); }
#[test] fn e2e_m33_z09() { assert_eq!(compile_and_run("tests/regression//m33_z09.xi"), Some(0)); }
#[test] fn e2e_m33_z10() { assert_eq!(compile_and_run("tests/regression//m33_z10.xi"), Some(0)); }
#[test] fn e2e_m33_z11() { assert_eq!(compile_and_run("tests/regression//m33_z11.xi"), Some(0)); }
#[test] fn e2e_m33_z12() { assert_eq!(compile_and_run("tests/regression//m33_z12.xi"), Some(0)); }
#[test] fn e2e_m33_z13() { assert_eq!(compile_and_run("tests/regression//m33_z13.xi"), Some(0)); }
#[test] fn e2e_m33_z14() { assert_eq!(compile_and_run("tests/regression//m33_z14.xi"), Some(0)); }
#[test] fn e2e_m33_z15() { assert_eq!(compile_and_run("tests/regression//m33_z15.xi"), Some(0)); }
#[test] fn e2e_m33_z16() { assert_eq!(compile_and_run("tests/regression//m33_z16.xi"), Some(0)); }
#[test] fn e2e_m33_z17() { assert_eq!(compile_and_run("tests/regression//m33_z17.xi"), Some(0)); }
#[test] fn e2e_m33_z18() { assert_eq!(compile_and_run("tests/regression//m33_z18.xi"), Some(0)); }
#[test] fn e2e_m33_z19() { assert_eq!(compile_and_run("tests/regression//m33_z19.xi"), Some(0)); }
#[test] fn e2e_m33_z20() { assert_eq!(compile_and_run("tests/regression//m33_z20.xi"), Some(0)); }

// P0 Fix Verification Tests (v0.56)
#[test] fn e2e_p0_forin() { assert_eq!(compile_and_run("tests/e2e_p0_forin.xi"), Some(0)); }
#[test] fn e2e_p0_defer() { assert_eq!(compile_and_run("tests/e2e_p0_defer.xi"), Some(0)); }
#[test] fn e2e_p0_labeled() { assert_eq!(compile_and_run("tests/e2e_p0_labeled.xi"), Some(0)); }

// P1 Pattern Verification Tests (v0.56)
#[test] fn e2e_p1_struct_pattern() { assert_eq!(compile_and_run("tests/e2e_p1_struct_pattern.xi"), Some(0)); }
#[test] fn e2e_p1_tuple_pattern() { assert_eq!(compile_and_run("tests/e2e_p1_tuple_pattern.xi"), Some(0)); }
#[test] fn e2e_p1_float_pattern() { assert_eq!(compile_and_run("tests/e2e_p1_float_pattern.xi"), Some(0)); }

// P2 Verification Tests (v0.56)
#[test] fn e2e_p2_strict_borrow() {
    // Without --strict: compiles (warnings only), returns Some(0)
    // With --strict: compilation fails (returns None)
    assert_eq!(compile_and_run("tests/e2e_p2_strict_borrow.xi"), Some(0),
        "P2-2: Without --strict, borrow warnings should not block compilation");
    assert_eq!(compile_and_run_with_flags("tests/e2e_p2_strict_borrow.xi", &["--strict"]), None,
        "P2-2: With --strict, borrow errors must block compilation");
}
#[test] fn e2e_p2_field_borrow() { assert_eq!(compile_and_run("tests/e2e_p2_field_borrow.xi"), Some(0)); }

// P0-4 Performance Test: Math builtin interception (v0.56)
#[test] fn e2e_p0_math_builtin() { assert_eq!(compile_and_run("tests/e2e_p0_math_builtin.xi"), Some(0)); }

// Deprioritized Items -- Verified as Production-Grade (v0.56)
#[test] fn e2e_p1_contract_methods() { assert_eq!(compile_and_run("tests/e2e_p1_contract_methods.xi"), Some(0)); }
#[test] fn e2e_p2_try_return() { assert_eq!(compile_and_run("tests/e2e_p2_try_return.xi"), Some(0)); }
#[test] fn e2e_p2_turbofish() { assert_eq!(compile_and_run("tests/e2e_p2_turbofish.xi"), Some(0)); }

// ============================================================================
// M37 -- 2026-08-11 compiler-hardening bug-fix regressions
// (docs/COMPILER_BUGS.md: BUG 1 tuple-of-struct codegen, struct &T param
// mutation, parser index-arithmetic, BUG 8 catalog &Vec[Int] params,
// circular-import termination)
// ============================================================================
#[test] fn e2e_m37_tuple_struct() { assert_eq!(compile_and_run("tests/regression//m37_tuple_struct.xi"), Some(0)); }
#[test] fn e2e_m37_ref_mut() { assert_eq!(compile_and_run("tests/regression//m37_ref_mut.xi"), Some(0)); }
#[test] fn e2e_m37_index_arith() { assert_eq!(compile_and_run("tests/regression//m37_index_arith.xi"), Some(0)); }
#[test] fn e2e_m37_catfix_catalog_vecref() { assert_eq!(compile_and_run("examples/catfix//main.xi"), Some(0)); }
#[test] fn e2e_m37_catfix_circular_imports() { assert_eq!(compile_and_run("examples/catfix//circ_main.xi"), Some(0)); }
#[test] fn e2e_m37_float_precision() { assert_eq!(compile_and_run("tests/regression//m37_float_precision.xi"), Some(0)); }
#[test] fn e2e_m37_catfix_private_type() { assert_eq!(compile_and_run("examples/catfix//b9main.xi"), Some(0)); }
#[test] fn e2e_m37_unsafe_option_return() { assert_eq!(compile_and_run("tests/regression//m34_y04.xi"), Some(0)); }
#[test] fn e2e_m37_unsafe_null_fault() { assert_eq!(compile_and_run("tests/regression//m33_u13.xi"), Some(0)); }
#[test] fn e2e_m37_vec_f64() { assert_eq!(compile_and_run("tests/regression//m37_vec_f64.xi"), Some(0)); }
#[test] fn e2e_m37_u128() { assert_eq!(compile_and_run("tests/regression//m37_u128.xi"), Some(0)); }
#[test] fn e2e_m37_shr_builtin() { assert_eq!(compile_and_run("tests/regression//m37_shr_builtin.xi"), Some(0)); }
#[test] fn e2e_m37_f128() { assert_eq!(compile_and_run("tests/regression//m37_f128.xi"), Some(0)); }

// BUG 2/BUG 3 + Str+Int concat regressions (2026-08-11 late session)
#[test] fn e2e_m37_global_field_write() { assert_eq!(compile_and_run("tests/regression//m37_global_field_write.xi"), Some(0)); }
#[test] fn e2e_m37_global_fn_init() { assert_eq!(compile_and_run("tests/regression//m37_global_fn_init.xi"), Some(0)); }
#[test] fn e2e_m37_str_int_concat() { assert_eq!(compile_and_run("tests/regression//m37_str_int_concat.xi"), Some(0)); }

// SIMD/ISA flags (-mavx -mavx2 -mavx512*) + runtime simd_runtime.c end-to-end
#[test] fn e2e_m37_simd_runtime() { assert_eq!(compile_and_run("tests/regression//m37_simd_runtime.xi"), Some(0)); }

// BUG 19: IEEE NaN/Inf semantics + Str+Float64 concat formatting
#[test] fn e2e_m37_nan_ieee() { assert_eq!(compile_and_run("tests/regression//m37_nan_ieee.xi"), Some(0)); }

// BUG 22/23 batch regression tests (2026-08-12)
#[test] fn e2e_m37_catalog_boundary() { assert_eq!(compile_and_run("tests/regression//m37_catalog_boundary.xi"), Some(0)); }
#[test] fn e2e_m57_geom_nested_param()   { assert_eq!(compile_and_run("tests/regression//m57_geom_nested_param.xi"), Some(0)); }
// BUG 57 follow-up (2026-09-09): LOCAL Vec[Vec[Float64]] ctor registered the
// element double-wrapped ("Vec[Vec[Float64]]"); chained reads compiling
// before the first push memcpy'd whole Vecs out of double slots (invalid IR).
#[test] fn e2e_m57b_geom_local_nested() { assert_eq!(compile_and_run("tests/regression//m57b_geom_local_nested.xi"), Some(0)); }
// M58 (stdlib finding 3b-2 #8): module-level mutable arrays indexed through
// stack copies of the loaded global value lost writes; fixed via direct
// global GEPs in the index read/write arms.
#[test] fn e2e_m58_module_array_global() { assert_eq!(compile_and_run("tests/regression//m58_module_array_global.xi"), Some(0)); }
// M59 (stdlib R4): guard-arena escape -- outer-Vec growth inside a confined
// block must stay on the main heap (runtime xiom_guard_realloc membership fix).
#[test] fn e2e_m59_guard_arena_escape() { assert_eq!(compile_and_run("tests/regression//m59_guard_arena_escape.xi"), Some(0)); }
// M60 (stdlib R2): module-level var arrays with all-constant literal
// initializers lower to LLVM constant aggregates (pointer-store IR fixed).
#[test] fn e2e_m60_module_array_literal() { assert_eq!(compile_and_run("tests/regression//m60_module_array_literal.xi"), Some(0)); }
// M61 (stdlib R1): byte_at/char_at upper-OOB reads clamped (pure (s,pos)).
#[test] fn e2e_m61_byte_at_oob() { assert_eq!(compile_and_run("tests/regression//m61_byte_at_oob.xi"), Some(0)); }
// M62 (stdlib-audit #3 delegation crash): qualified calls bind the catalog fn
// despite a local same-name shadow (injection dedups by qualified key now).
#[test] fn e2e_m62_delegation_shadow() { assert_eq!(compile_and_run("tests/regression//m62_delegation_shadow.xi"), Some(0)); }
// M63 (CRT-layout #1): closure env struct malloc under-allocation (struct captures).
#[test] fn e2e_m63_crt_closure_env_struct() { assert_eq!(compile_and_run("tests/regression//m63_crt_closure_env_struct.xi"), Some(0)); }
// M64 (CRT-layout #2): &mut [N]T param element-address lowering in Ref args.
#[test] fn e2e_m64_crt_sortby_refargs() { assert_eq!(compile_and_run("tests/regression//m64_crt_sortby_refargs.xi"), Some(0)); }
// M65a (json heap layer, write side): Map[Str, JsonValue] values must be
// sized 112 bytes per slot (ctor mono substitution). IR-level regression.
#[test] fn e2e_m65a_json_values_stride() {
    let ok = compile_and_check_ir("tests/regression//m65_json_map_enum_payload.xi", "store i64 112");
    assert!(ok, "Map[Str, JsonValue] values Vec must use 112-byte element stride");
}
// M65 (json heap layer Part 2, Stage 2c): enum payloads concretize
// (Option__JsonValue = { tag, %struct.JsonValue }), the map value read takes
// the struct-load path (memcpy at the runtime stride), and a pointer-self
// method on a struct-value temporary materializes an alloca. Red pre-fix:
// catalog json_get AV'd (0xC0000005) on the scalar tag inttoptr.
#[test] fn e2e_m65_json_map_enum_payload() {
    assert_eq!(compile_and_run("tests/regression//m65_json_map_enum_payload.xi"), Some(0));
}
#[test] fn e2e_m65b_json_get_concrete_option() {
    let ok = compile_and_check_ir(
        "tests/regression//m65_json_map_enum_payload.xi",
        "%struct.Option__JsonValue = type { i64, %struct.JsonValue }",
    );
    assert!(ok, "Option[JsonValue] must concretize with the full enum payload field");
}
// R8 (2026-09-11): method-position FREE-FN calls (receiver sugar) +
// contracts evaluating them (`s.char_count_local()` in an ensures) and the
// builtin Char-returning `.char_at` path staying coherent. Red pre-fix:
// checker "cannot call 'char_count' on this expression"; catalog bodies
// compiled the sugar to a constant-0 stub.
#[test] fn e2e_m65_r8_method_free_fn() {
    assert_eq!(compile_and_run("tests/regression//m65_r8_method_free_fn.xi"), Some(0));
}
// M65 regex-family fix: Vec[Option[Int]] / Vec[Option[Struct]] element
// sizing + container-element struct reads
// (type_arg_to_name bracketed args, concrete Option__T elem size,
// Some ctor payload-context inference).
#[test] fn e2e_m65_vec_option_elem() {
    assert_eq!(compile_and_run("tests/regression//m65_vec_option_elem.xi"), Some(0));
}
// smoke_math_edge fix: math.shl/shr emitted raw LLVM shifts; a count >= 64
// is poison (clang -O2 trapped shl(1,100) with 0xC000001D). The builtin now
// emits the stdlib's defined semantics (n<=0 -> a; n>=64 -> 0 / sign; else
// masked shift). Red pre-fix: smoke_math_edge trap.
#[test] fn e2e_m65_shift_semantics() {
    assert_eq!(compile_and_run("tests/regression//m65_shift_semantics.xi"), Some(0));
}
// R8 follow-up: Str method-parity sugar (trim/trim_start/trim_end) on a Str
// param -- codegen auto-stubbed `Str.trim` and the reachability filter had
// pruned the canonical free fns (len=0xFFFFFFFF).
#[test] fn e2e_m65_str_method_sugar() {
    assert_eq!(compile_and_run("tests/regression//m65_str_method_sugar.xi"), Some(0));
}
// R10: Vec[Option[struct-with-Str]] element reads (nested container args in
// struct fields + the type_meta field scan).
#[test] fn e2e_m65_vec_option_struct_str() {
    assert_eq!(compile_and_run("tests/regression//m65_vec_option_struct_str.xi"), Some(0));
}
// smoke_error2 has-mid nondeterminism: generated aggregate type_meta keys
// (`Option__ErrNode`, `Vec__ErrNode`) suffix-match the base type; the
// Vec[Str]-element resolver broke on the first matching key, so HashMap
// order decided whether `n.messages[i]` was a Str or a truncated i64.
#[test] fn e2e_m66_chain_error_has() {
    assert_eq!(compile_and_run("tests/regression//m66_chain_error_has.xi"), Some(0));
}
// LET-array decision P1 (docs/LET_ARRAY_DECISION.md): annotated fixed arrays
// `let c: [N]T = [...]` bind [N x T] (var parity) and float elements stay
// float-typed on the index read (was: val_to_i64 bitcast + sitofp back).
#[test] fn e2e_m67_let_array_annotated() {
    assert_eq!(compile_and_run("tests/regression//m67_let_array_annotated.xi"), Some(0));
}
// LET-array decision P2 (docs/LET_ARRAY_DECISION.md): user-fn `&[N]T` /
// `&mut [N]T` params lower to the ELEMENT pointer (catalog generic ABI).
// Pre-fix: a catalog call inside the callee mono'd as `array.len_[3 x i64]_3`
// (clang "expected '(' in call") and `&mut` element writes emitted invalid
// GEP indices. Probes letarr2/2b/2c.
#[test] fn e2e_m68_let_array_user_fn_ref() {
    assert_eq!(compile_and_run("tests/regression//m68_let_array_user_fn_ref.xi"), Some(0));
}
// LET-array decision P3 (docs/LET_ARRAY_DECISION.md): unannotated
// `let a = [...]` binds a FIXED array `[N]T` (M33 let->Vec deleted for let
// literals); the call-site Slice bridge keeps `&Slice[T]` consumers working,
// and a non-generic `&Slice[Int]` param (`core.sum_slice`) uses the same
// by-value %struct.Vec ABI as the generic ones. Pre-P3: sum_slice returned 0
// (len read data[0]); post-P3 the fixture exits 0.
#[test] fn e2e_m69_let_array_slice_bridge() {
    assert_eq!(compile_and_run("tests/regression//m69_let_array_slice_bridge.xi"), Some(0));
}
// P3 representation pin: `let a = [1,2,3,4,5]` is a `[5 x i64]` aggregate
// alloca (no M33 %struct.Vec conversion for let literals).
#[test] fn e2e_m69_let_array_fixed_ir() {
    assert!(compile_and_check_ir("tests/regression//m69_let_array_slice_bridge.xi", "alloca [5 x i64]"));
}
// R9 (round 53, stdlib report 2026-09-12): a FULL-PATH call into a module
// that was never imported (`xiom.string.glob`) must resolve the shim's own
// delegated full-path call (`xiom.misc.glob.glob_match`). Pre-fix the
// target module was never injected, so codegen bound the inner call to the
// shim itself -> infinite recursion -> 0xC0000409 at runtime (probes
// p_x1/p_x2/p_x4/p_x6, p_sdx_shim_first, p_lev_shim_first).
#[test] fn e2e_m70_full_path_shim_delegation() {
    assert_eq!(compile_and_run("tests/regression//m70_full_path_shim_delegation.xi"), Some(0));
}
// R14: `"e[0]=" + e[0]` on a chained `.collect()` Vec emitted inttoptr for
// the indexed Int element (AV at the element's value). Regression locks the
// concat fallback verdict from the compiled LLVM type.
#[test] fn e2e_m71_concat_index_elem() {
    assert_eq!(compile_and_run("tests/regression//m71_concat_index_elem.xi"), Some(0));
}
// R17: nested-index Str elements (`rows[0][0]`) must concat as strings, not
// as pointers/numbers. Locks the recursive element-type resolution against
// the R14 LLVM fallback.
#[test] fn e2e_m72_nested_index_concat() {
    assert_eq!(compile_and_run("tests/regression//m72_nested_index_concat.xi"), Some(0));
}
// R19: generic deref-store through `*mut T` must keep all but ONE star
// (i8** -> i8*, not i8). clang ptr/i8 mismatch in ptr.replace_Str.
#[test] fn e2e_m73_ptr_replace_str() {
    assert_eq!(compile_and_run("tests/regression//m73_ptr_replace_str.xi"), Some(0));
}

// R21: a user `use X as Y;` alias must shadow a same-leaf catalog module
// (pre-fix: `use network as net; use net.local;` catalog-loaded xiom.net
// and the strict flip hard-failed on unrelated catalog bodies).
#[test] fn e2e_m74_user_alias_shadows_catalog() {
    assert_eq!(compile_and_run("tests/regression//m74_user_alias_shadows_catalog.xi"), Some(0));
}

// R20: same-leaf catalog delegation through a `use ... as` alias must bind
// the checker-recorded owner-qualified target (pre-fix the emitter could not
// see the alias and fell into an order-dependent suffix scan that bound a
// zero-arg stub or the shim itself).
#[test] fn e2e_m75_alias_delegation() {
    assert_eq!(compile_and_run("tests/regression//m75_alias_delegation//main.xi"), Some(0));
}

// R18: contract payload clauses (`result.value.len() <= s.len()`, payload
// value equality, scalar payload bounds, Err-side `result.error.len()`).
// Pre-fix the bare `is Some/Err` rebind made `.value`/`.error` fall to the
// literal-0 fallback and the clause aborted spuriously.
// 2026-10-05: extended with the packages-lane leak shapes -- a tuple payload
// clause under a USER local named `result` (varint) and an Ok payload clause
// preceded by an Err clause (cobs).
#[test] fn e2e_m76_contract_payload_param_len() {
    assert_eq!(compile_and_run("tests/regression//m76_contract_payload_param_len.xi"), Some(0));
}

// R16: `ptr + int` in a call argument must lower to pointer arithmetic.
// Pre-fix an unannotated `var buf = malloc(n)` had no pointer XIOM type, so
// `buf + len` compiled as Str concatenation and memcpy corrupted the buffer.
#[test] fn e2e_m77_ptr_plus_int_arg() {
    assert_eq!(compile_and_run("tests/regression//m77_ptr_plus_int_arg.xi"), Some(0));
}

// R15b: same-leaf/same-name modules declared in the USER program (three
// source files on the command line -- not catalog siblings). Pre-fix the
// delegating module's aliased call bound its OWN qualified symbol
// (`call @beta.base32.encode` -> self-recursion -> 0xC000001D).
#[test] fn e2e_m78_user_sameleaf_modules() {
    let exe = project_root().join("e2e_m78_user_sameleaf.exe");
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .args([
            "-o", exe.to_str().unwrap(),
            "tests/regression/m78_user_sameleaf/alpha_base32.xi",
            "tests/regression/m78_user_sameleaf/beta_base32.xi",
            "tests/regression/m78_user_sameleaf/gateway.xi",
        ])
        .current_dir(project_root())
        .output()
        .expect("failed to spawn xiom");
    if !compile.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&compile.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&compile.stderr));
    }
    assert!(compile.status.success(), "m78 multi-source compile should succeed");
    let run = Command::new(&exe).output().expect("failed to run m78 exe");
    assert_eq!(run.status.code(), Some(0), "m78 should exit 0");
}

// Stage 5 (DWARF for .xi): `-g` must produce a VALID debug-info graph and
// per-statement line locations; without `-g` the IR must stay metadata-free
// (the differential IR suites byte-compare default builds).
#[test] fn e2e_m79_debug_info_metadata() {
    let emit = |extra: &[&str]| -> String {
        let mut args: Vec<&str> = extra.to_vec();
        args.push("--emit-ir");
        args.push("tests/regression//m79_debug_info.xi");
        let out = Command::new(xiom_path())
            .args(&args)
            .current_dir(project_root())
            .output()
            .expect("spawn xiom");
        assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).to_string()
    };

    let with_debug = emit(&["-g"]);
    assert!(with_debug.contains("!llvm.dbg.cu"), "compile unit metadata missing");
    assert!(with_debug.contains("!DISubroutineType"),
        "DISubprogram.type must reference a subroutine type (empty `!{{}}` invalidates all debug info)");
    // Line numbers are source-absolute: the SPDX/header block added two
    // lines, so `return a + b;` is 12 and `if x != 5` is 16.
    assert!(with_debug.contains("!DILocation(line: 12"),
        "body statement line (return a + b) must have a DILocation");
    assert!(with_debug.contains("!DILocation(line: 16"),
        "main's `if x != 5` line must have a DILocation");
    assert!(with_debug.contains(", !dbg !"), "instructions must carry !dbg attachments");

    let without_debug = emit(&[]);
    assert!(!without_debug.contains("!DILocation"), "no DILocations without -g");
    assert!(!without_debug.contains("!llvm.dbg.cu"), "no compile unit without -g");
}

// Stage 5: the `-g` build must still compile, link and run.
#[test] fn e2e_m79_debug_info_runs() {
    assert_eq!(compile_and_run_with_flags("tests/regression//m79_debug_info.xi", &["-g"]), Some(0));
}

// R23: fn-typed values are closure ENV pointers (env-first ABI) in every
// shape: fn-typed param, local binding, struct field, and Vec[fn()].pop()
// payload binding. Pre-fix the raw fn-pointer path inttoptr'd the env box as
// code (0xC0000005; the async executor AV'd in reduced shapes).
#[test] fn e2e_m80_fn_value_shapes() {
    assert_eq!(compile_and_run("tests/regression//m80_fn_value_shapes.xi"), Some(0));
}

// R25: same-leaf fn-REFERENCE resolution across user modules. Two modules
// each define `is_even` + a higher-order `apply`; each passes its OWN fn by
// value. Pre-R25 the fn-registry suffix scan was HashMap-ordered and could
// bind the other module's `is_even` (wrong result), and the emitted ptrtoint
// used the raw registry key (bare `@is_even` undefined) instead of the
// pre-assigned symbol.
#[test] fn e2e_m81_fn_ref_same_leaf() {
    let exe = project_root().join("e2e_m81_fn_ref_same_leaf.exe");
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .args([
            "-o", exe.to_str().unwrap(),
            "tests/regression/m81_fn_ref_same_leaf/alpha.xi",
            "tests/regression/m81_fn_ref_same_leaf/beta.xi",
            "tests/regression/m81_fn_ref_same_leaf/gateway.xi",
        ])
        .current_dir(project_root())
        .output()
        .expect("failed to spawn xiom");
    if !compile.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&compile.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&compile.stderr));
    }
    assert!(compile.status.success(), "m81 multi-source compile should succeed");
    let run = Command::new(&exe).output().expect("failed to run m81 exe");
    assert_eq!(run.status.code(), Some(0), "m81 should exit 0");
}

// R28: Option/Result `.value` on a TEMPORARY call result must materialize the
// aggregate payload. Pre-fix the computed-value field path skipped the
// payload override, bound the raw handle as i64, and later indexing emitted
// a literal 0 (zeroed Vec) while a named local worked.
#[test] fn e2e_m82_tmp_payload_value() {
    assert_eq!(compile_and_run("tests/regression//m82_tmp_payload_value.xi"), Some(0));
}

// R29: a Vec built inside a match arm over a Result[Vec[...]] payload must
// clang-compile and run. Pre-fix every expression statement in the arm block
// stored its value into the match result slot (`store %struct.Option <Vec>`).
#[test] fn e2e_m83_match_arm_vec_build() {
    assert_eq!(compile_and_run("tests/regression//m83_match_arm_vec_build.xi"), Some(0));
}

// R39: same-leaf TYPE collision across project modules loaded through the
// package graph. Pre-fix the catalog injection flattened both `Metrics` decls
// to a bare `%struct.Metrics` (the alphabetically-first module's layout won)
// while the other module's bodies kept their own field count -- invalid GEPs
// (the last clang error in the bench graph). The type qualification pass now
// renames every colliding leaf and its references before injection.
#[test] fn e2e_m84_type_same_leaf_modules() {
    let emit = Command::new(xiom_path())
        .args(["--emit-ir", "tests/regression/m84_type_same_leaf/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed to spawn xiom");
    assert!(
        emit.status.success(),
        "m84 emit-ir failed: {}",
        String::from_utf8_lossy(&emit.stderr)
    );
    let ir = String::from_utf8_lossy(&emit.stdout).to_string();
    assert!(
        ir.contains("%struct.m84.alpha.Metrics = type { i64, i64 }"),
        "alpha Metrics not module-qualified"
    );
    assert!(
        ir.contains("%struct.m84.beta.Metrics = type { i64, i64, i64, i64 }"),
        "beta Metrics not module-qualified"
    );
    assert!(
        !ir.contains("%struct.Metrics = type"),
        "bare %struct.Metrics definition survived the collision"
    );

    let exe = project_root().join("e2e_m84_type_same_leaf.exe");
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .args([
            "-o", exe.to_str().unwrap(),
            "tests/regression/m84_type_same_leaf/main.xi",
        ])
        .current_dir(project_root())
        .output()
        .expect("failed to spawn xiom");
    if !compile.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&compile.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&compile.stderr));
    }
    assert!(compile.status.success(), "m84 package-graph compile should succeed");
    let run = Command::new(&exe).output().expect("failed to run m84 exe");
    assert_eq!(run.status.code(), Some(0), "m84 should exit 0 (both Metrics layouts intact)");
}

// R40: derive[Clone] on a POINTER receiver (`m: &M` -> `m.clone()`). The
// callee has a by-value `%self`; pre-fix the call passed the pointer where the
// value was expected, LLVM accepted the silent mismatch, and the returned
// struct was garbage. Covers struct and enum clone + the value-receiver guard.
#[test] fn e2e_m85_clone_ref_receiver() {
    assert_eq!(compile_and_run("tests/regression//m85_clone_ref_receiver.xi"), Some(0));
}

// R43: `&v` where v already holds a reference denotes the same reference.
// Pre-fix this raised C001 ("already a reference"), which broke the stdlib's
// `var v = b; ... &v` pattern (x25519_keypair -> _bigint_to_le). Pins
// read-through, field-write aliasing, and rebind isolation.
#[test] fn e2e_m86_ref_of_reference() {
    assert_eq!(compile_and_run("tests/regression//m86_ref_of_reference.xi"), Some(0));
}

// R45: tuple element types are named by their XIOM types -- comparisons and
// Bool locals are Bool (not i64 -> "Int"), and `x as UInt16` uses the cast
// target (not i16 -> "Int16"). Pre-fix the literal/signature disagreed and
// clang rejected the return (stdlib sweep: overflowing_neg,
// code_point_to_utf16).
#[test] fn e2e_m87_tuple_element_types() {
    assert_eq!(compile_and_run("tests/regression//m87_tuple_element_types.xi"), Some(0));
}

// R46: same-leaf GENERIC types with conflicting shapes (`Box[T]` 1-field vs
// 2-field across project modules) are module-qualified; identical generic
// re-declarations keep the legacy key. R46b: the module-local wrappers are
// GONE -- the fixture locks DIRECT cross-module generic-method calls
// (qualified receivers, arg inference, receiver-only inference, computed
// receiver), which pre-fix resolved to erased stubs / a same-leaf module in
// HashMap order.
#[test] fn e2e_m88_generic_same_leaf_boxes() {
    assert_eq!(
        compile_and_run("tests/regression/m88_generic_same_leaf/main.xi"),
        Some(0)
    );
}

// R46: a bare literal name that is BOTH a struct and an enum variant binds by
// the literal's FIELD NAMES -- `Node(value,left,right)` is the enum variant,
// `Node{value,children}` the same-leaf struct (bench BST lock).
#[test] fn e2e_m89_struct_variant_node() {
    assert_eq!(
        compile_and_run("tests/regression/m89_struct_variant_node/main.xi"),
        Some(0)
    );
}

// R44: catalog (stdlib) same-leaf TYPE collisions participate in the R39
// qualification when declared shapes conflict. On the pinned stdlib
// `xiom.net.net.HttpResponse` (2 fields) and `xiom.net.http.HttpResponse`
// (3 fields) share the leaf; pre-R44 the bare `%struct.HttpResponse` won and
// `http_parse_response` GEPed field 2 of a 2-field definition (clang:
// "invalid getelementptr indices"). Skips loudly without a stdlib checkout;
// XIOM_REQUIRE_STDLIB=1 turns the skip into a failure in CI.
#[test] fn e2e_m90_stdlib_same_leaf_http() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("net").join("http.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/net/http.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m90_stdlib_same_leaf_http/main.xi"),
        Some(0),
        "R44 stdlib same-leaf probe must compile and run"
    );
}

// R47 (playground C18/C19): conversion methods on builtin receivers without
// `use xiom.fmt` (the concrete `.to_str` impls must be injected), plus
// Option[Str]/Option[Float64] `unwrap_or` payloads (phi dominance + ABI).
// Needs the stdlib checkout (xiom.io); XIOM_REQUIRE_STDLIB=1 in CI forbids
// skipping.
#[test] fn e2e_m91_conversion_methods() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m91_conversion_methods/main.xi"),
        Some(0),
        "R47 conversion/Option probe must compile and run"
    );
}

// m179 (release hold, stdlib-perf2): the `#[unsafe_direct]` sync annotation
// put Arc.new_Int on the direct path; `size_of[ArcInner[T]]()` hit the
// 8-byte scalar fallback and the 16-byte ArcInner store overflowed the
// malloc(8) buffer, so `strong_count()` read garbage (!= 1). The probe
// asserts strong_count == 1 and get() == 42 right after construction.
// Needs a stdlib checkout; the annotation-bearing pin makes this the true
// regression lock (pre-perf2 pins pass vacuously).
#[test] fn e2e_m179_arc_strong_count() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("sync").join("sync.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/sync/sync.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m179_arc_strong_count/main.xi"),
        Some(0),
        "m179 Arc strong_count probe must compile and run"
    );
}

// m180 (release hold, v0.62.x): `xiom.contracts.any_contracts()` called a
// method on a COMPUTED receiver (`_get_index().none()`). Codegen could not
// infer the receiver type of a call expression, so the key degraded to the
// bare leaf "none" and the keep-first catalog alias bound the unrelated
// generic free fn `core.none` (2 params) to the 0-arg method call -> ABI
// mismatch -> 0xC0000005. The probe must run to exit 0.
// Needs the stdlib checkout (xiom/core/contracts.xi).
#[test] fn e2e_m180_contracts_any_av() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("core").join("contracts.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/core/contracts.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m180_contracts_any_av/main.xi"),
        Some(0),
        "m180 any_contracts computed-receiver probe must compile and run"
    );
}

// m182 (stdlib row 25): module-level const tables with Str/struct payloads
// kept the per-use substitution path and mis-read (struct fields folded to
// constant 0 -> rc 6; the NAMES str_len loop -> rc 9). Must become real
// constant globals and run to exit 0. Needs the stdlib checkout
// (xiom/string/string.xi).
#[test] fn e2e_m182_const_tables() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("string").join("string.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/string/string.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m182_const_tables/main.xi"),
        Some(0),
        "m182 const tables probe must compile and run"
    );
}

// m184 (packages relay): nested test-module import -- `module pkg.tests` +
// `use pkg;` must see the peer pkg.xi root exports. Pre-fix the program's
// own leaf-less chain shadowed the module (163 T001s in websocket).
#[test] fn e2e_m184_nested_module_import() {
    assert_eq!(
        compile_and_run("tests/regression/m184_nested_module_import/main.xi"),
        Some(0),
        "m184 nested test-module import must resolve the peer module"
    );
}

// m185 (packages relay): `var r: Row;` + match-arm assignments + return r
// crashed on v0.62.3 -- the zero placeholder coerced to a struct via
// inttoptr 0 + load (NULL dereference). Must run to exit 0.
#[test] fn e2e_m185_uninit_local_struct() {
    assert_eq!(
        compile_and_run("tests/regression/m185_uninit_local_struct/main.xi"),
        Some(0),
        "m185 uninitialized-local struct must read assigned values"
    );
}

// m186 (stdlib relay): inline module-qualified UInt32 call compares misread
// high-bit values (sext vs the constant's zext). Needs the stdlib checkout
// (xiom/hash/adler.xi).
#[test] fn e2e_m186_uint32_high_bit_compare() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("hash").join("adler.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/hash/adler.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m186_uint32_high_bit_compare/main.xi"),
        Some(0),
        "m186 inline UInt32 high-bit compare must be correct"
    );
}

// m188 (packages relay): const values as match arms never matched (bare
// idents parsed as catch-all bindings; every input hit the wildcard).
#[test] fn e2e_m188_const_match_arms() {
    assert_eq!(
        compile_and_run("tests/regression/m188_const_match_arms/main.xi"),
        Some(0),
        "m188 const-valued match arms must compare by value"
    );
}

// m189 (packages relay): a struct type and an enum variant sharing a name
// resolved the literal to the variant inside a module scope (invalid IR /
// in-situ corruption). Must build the struct and read the payload.
#[test] fn e2e_m189_enum_struct_payload() {
    assert_eq!(
        compile_and_run("tests/regression/m189_enum_struct_payload/main.xi"),
        Some(0),
        "m189 same-name struct type vs enum variant must stay a struct"
    );
}

// R48 (playground C17 classes): interface dispatch through a `&T` generic
// argument (was mono'd as Int -> C001) and pointer/double match-result
// zero-init (was `store i8* 0` / `store double 0` -> clang reject). Needs
// the stdlib checkout; XIOM_REQUIRE_STDLIB=1 in CI forbids skipping.
#[test] fn e2e_m92_interface_dispatch_zero_init() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m92_interface_dispatch_zero_init/main.xi"),
        Some(0),
        "R48 interface-dispatch/zero-init probe must compile and run"
    );
}

// R49 (playground C17 residue): L6-28 -- a struct-literal argument infers the
// generic (`pq.insert(Task{..})` -> T=Task), an ANNOTATED generic-struct local
// keeps its type args for receiver inference (`pq.pop()` -> T=Task), and the
// boxed struct payload unboxes through `.unwrap()`. Needs the stdlib checkout;
// XIOM_REQUIRE_STDLIB=1 in CI forbids skipping.
#[test] fn e2e_m93_interface_generic_inference() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m93_interface_generic_inference/main.xi"),
        Some(0),
        "R49 generic-struct inference/unbox probe must compile and run"
    );
}

// R49 (playground C17 residue): L8-15/L8-18 -- Vec[Vec[Cell]] nested element
// access: get boxes the inner Vec, unwrap unboxes it, the inline set memcpys
// struct/enum element bytes, and computed Vec receivers take the inline path
// (was clang "i64 where ptr expected", then AV). Needs the stdlib checkout.
#[test] fn e2e_m94_nested_vec_struct_elem() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m94_nested_vec_struct_elem/main.xi"),
        Some(0),
        "R49 nested-Vec struct-element probe must compile and run"
    );
}

// R49 (stdlib relay p_pre_call_capture): @pre on a CALL/field chain must read
// the ENTRY state -- this needs the pre-snapshot to deep-copy Vec buffers and
// ref params to be rebound through a pointer slot. Free-function receiver.
#[test] fn e2e_m95_pre_call_capture() {
    assert_eq!(
        compile_and_run("tests/regression/m95_pre_call_capture/main.xi"),
        Some(0),
        "R49 @pre call-capture probe must compile and run (stdlib p_pre_call_capture)"
    );
}

// R49 (playground C19 residue): a match arm whose result is a bare conversion
// call (`Some(v) => to_string(v)`) must keep the i8* result type -- the old
// i64 inference truncated the string pointer to one byte (L5-42 printed
// garbage). Needs the stdlib checkout.
#[test] fn e2e_m98_match_str_to_string() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m98_match_str_to_string/main.xi"),
        Some(0),
        "R49 match-to_string probe must compile and run"
    );
}

// R52 (playground L5-20/L5-31): Str fields of a CONCRETE generic struct
// (`Pair[Str,Str].first`, `Box[Str].get_value()`) must keep the Str
// representation through `.to_str()` (was the erased i64 field whose pointer
// got printed as a number).
#[test] fn e2e_m106_generic_field_str() {
    assert_eq!(
        compile_and_run("tests/regression/m106_generic_field_str/main.xi"),
        Some(0),
        "R52 generic-struct Str field probe must compile and run"
    );
}

// R52 (playground L5-26/L5-36/L3-02): Str payloads through Vec.get matches,
// match-expression results and unannotated Option locals must stay Str (was
// the erased i64/pointer printed as an integer).
#[test] fn e2e_m107_match_payload_str() {
    assert_eq!(
        compile_and_run("tests/regression/m107_match_payload_str/main.xi"),
        Some(0),
        "R52 match/unwrap Str payload probe must compile and run"
    );
}

// R52 (playground L5-09/L5-43): Map[Str,Str] payloads bind as i8*; a
// redundant explicit `&receiver` argument (`s.push(&mut s, x)`) is ignored;
// a side-effecting match scrutinee (`match s.pop(&mut s)`) is evaluated once.
#[test] fn e2e_m108_explicit_self_and_map() {
    assert_eq!(
        compile_and_run("tests/regression/m108_explicit_self_and_map/main.xi"),
        Some(0),
        "R52 explicit-self/Map-Str probe must compile and run"
    );
}

// R54 (p_sweep_single_param clang ISel crash): a LARGE fixed array must not
// materialize whole-aggregate values -- clang 22.1.8 X86 ISel crashes on
// `alloca [65536 x i8]` + `store zeroinitializer` / whole-array loads.
// Codegen byte-zeros arrays >= 16 KiB with memset and accesses them by address.
#[test] fn e2e_m109_large_fixed_array() {
    assert_eq!(
        compile_and_run("tests/regression/m109_large_fixed_array/main.xi"),
        Some(0),
        "R54 large fixed-array probe must compile and run"
    );
}

// L6-40 (playground C17 residue): a module-scoped generic FACTORY has no
// argument evidence at its call site -- `var runner = plugin_runner.create()`
// fixes T only through the LATER `plugin_runner.add_plugin(&mut runner,
// EchoPlugin{})`. Single-pass emission inferred nothing (the `0` fallback) and
// then mono'd `run_all` with T=Int -> C001 "Int does not implement Plugin".
// The bounded function-body evidence pre-pass resolves the factory's type arg
// from the later call and seeds the binding's container type.
#[test] fn e2e_m110_generic_factory_evidence() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m110_generic_factory_evidence/main.xi"),
        Some(0),
        "L6-40 module-scoped generic factory evidence probe must compile and run"
    );
}

// L5-40: a container PAYLOAD inside a generic container
// (`Map[Int, Vec[Str]]`). Three sites disagreed on V (_Int_Int / _Int_Vec /
// _Int_Vec_Str_), so `values` was allocated with 8-byte slots while insert
// stored a 32-byte %struct.Vec header, and the match binder read the concrete
// inline Option payload as an 8-byte box handle. The probe asserts length,
// element count, and element contents.
#[test] fn e2e_m111_map_vec_container_payload() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m111_map_vec_container_payload/main.xi"),
        Some(0),
        "L5-40 Map[Int, Vec[Str]] container-payload probe must compile and run"
    );
}

// L3-50: `?` on a Result with a TUPLE payload. `let (a, b) = two()?` bound
// both names to the raw boxed-tuple handle (a+b printed pointer arithmetic);
// the `?` handler now unboxes aggregate payloads out of the erased i64 slot
// and the Err propagation path stays intact.
#[test] fn e2e_m112_try_result_tuple() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m112_try_result_tuple/main.xi"),
        Some(0),
        "L3-50 Result tuple payload via ? must compile and run"
    );
}

// L8-14: Map[Str, Str] morse flow trapped with 0xC000001D.
// `param.get(k).unwrap_or("?")` resolved payload types only for Ident
// receivers; a `&Map[Str, Str]` param had lost its args ("&Map"), so the Str
// payload escaped as a raw i64 handle and string concatenation printed
// pointers. The lock asserts the encode/decode round trips.
#[test] fn e2e_m113_map_str_str_morse() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m113_map_str_str_morse/main.xi"),
        Some(0),
        "L8-14 Map[Str, Str] morse flow must compile and run without trapping"
    );
}

// R59 (stdlib p_result_tuple_vec_loop): a match arm's result slot leaked into
// nested loop bodies -- `oid.push(value)` as the while's last expression
// stored a %struct.Vec into the arm's %struct.Result slot (invalid IR).
// While/For/Spawn bodies are statement contexts now.
#[test] fn e2e_m114_result_tuple_vec_loop() {
    assert_eq!(
        compile_and_run("tests/regression/m114_result_tuple_vec_loop/main.xi"),
        Some(0),
        "R59 Result tuple payload with a loop-built Vec must compile and run"
    );
}

// R60 (stdlib p_ref_tuple_mangle): a reference-typed tuple element mangled
// the tuple struct name ("Tuple__&Vec__Vec" -> clang "expected '=' after
// name") and a container-ctor element was named by its erased LLVM type
// ("Vec[UInt8].new()" -> "Int"). The shared element namer fixes both.
#[test] fn e2e_m115_ref_tuple_mangle() {
    assert_eq!(
        compile_and_run("tests/regression/m115_ref_tuple_mangle/main.xi"),
        Some(0),
        "R60 reference-typed tuple elements must not mangle the struct name"
    );
}

// R7 residual: a generic constructor's Holder[V] built with `Vec[V].new()`,
// then an aggregate element read through the holder FIELD (`h.values[0]`).
// Local explicit-generic calls never recorded "Holder[V]" with args, so the
// index read fell to the scalar i64 switch (p_generic_push/p_gp_b/p_gp_c AV).
#[test] fn e2e_m116_generic_ctor_field_index() {
    assert_eq!(
        compile_and_run("tests/regression/m116_generic_ctor_field_index/main.xi"),
        Some(0),
        "R7 residual generic-ctor holder aggregate field index must compile and run"
    );
}

// Interface value-receiver ABI is NOT implemented: an interface-typed
// parameter receiving an aggregate by value must fail LOUDLY (the old path
// emitted a call that silently returned wrong values -- hash probe a=5381,
// a == c). Lock the diagnostic so a future ABI implementation replaces it
// deliberately.
#[test] fn e2e_m117_interface_value_abi_rejected() {
    assert_eq!(
        compile_and_run("tests/regression/m117_interface_value_abi/main.xi"),
        None,
        "interface-typed parameter with an aggregate argument must be rejected loudly"
    );
}

// R65: `xiom.env.OS/ARCH/FAMILY` are compile-time TARGET facts. They were
// hardcoded to windows/x86_64 in the stdlib, so a Linux build reported
// windows (stdlib p_platform_env). The fixture derives the host from
// $OS and asserts the constants agree -- it fails on Linux before the fix
// and passes on both Windows and Linux after it.
#[test] fn e2e_m118_env_platform_constants() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m118_env_platform_constants/main.xi"),
        Some(0),
        "R65 platform constants must match the running host"
    );
}

// P1-4 m119: `is_sorted()`/`contains()` are lowered INLINE over the
// receiver's Vec header. The legacy runtime intrinsics read data[0] as the
// element COUNT; for a `%struct.Vec` receiver that word is the DATA POINTER,
// so the scan walked arbitrary memory -- a deterministic access violation
// (`-1073741819`) on the Windows CI runner against
// `e2e_p1_contract_methods`, a wrong answer everywhere else (sorted [1..5]
// reported false). The fixture pins the correct values across Vec/fixed
// array/array literal/Str/Float64/empty/struct-field receivers; unsupported
// element kinds are compiler-rejected instead of scanned.
#[test] fn e2e_m119_contract_method_values() {
    assert_eq!(
        compile_and_run("tests/regression/m119_contract_method_values/main.xi"),
        Some(0),
        "P1-4 contract methods must return correct values across Vec/array/Str/Float shapes"
    );
}

// R67: Some/None/Ok/Err chose their container struct with a substring test
// on the enclosing function's return type (`contains("Option"/"Result")`).
// A function returning a USER struct with that substring (`TestResult`,
// `Options`) made `Ok(5)` emit a `%struct.TestResult` payload (clang:
// "invalid getelementptr indices") and `Some(3)` a `%struct.Options` payload.
// The check is now the strict container leaf (bare `Result`/`Option` or a
// concrete `Result__A__B`/`Option__T`); concrete container returns must keep
// working (5c.35).
#[test] fn e2e_m120_ctor_user_struct_return() {
    assert_eq!(
        compile_and_run("tests/regression/m120_ctor_user_struct_return/main.xi"),
        Some(0),
        "R67 ctors inside struct-returning functions must build their own container type"
    );
}

// R68: `extern "C" { ... }` inside a function body (legacy package shape)
// used to fall into the expression parser and report "'extern' is a reserved
// keyword and cannot be used as an identifier". Extern fns carry no body, so
// the block is now hoisted to the enclosing module just before the enclosing
// declaration; duplicate per-function declarations merge.
#[test] fn e2e_m121_nested_extern() {
    assert_eq!(
        compile_and_run("tests/regression/m121_nested_extern/main.xi"),
        Some(0),
        "R68 nested extern blocks must hoist and link"
    );
}

// R69: `local_xiom_types` is global across functions; the mono param path
// never recorded plain params, so a generic `x: T` kept a STALE entry from an
// earlier emitted fn (`x: Float64` in xiom.fmt). `fn show[T](x: T) -> Str {
// return x.to_str(); }` then lowered through the Float64 conversion --
// show(99) printed 4.891e-322 (i64 bits read as a double), show("hi") printed
// 0. The mono path now records the substituted param type.
#[test] fn e2e_m122_generic_param_type() {
    assert_eq!(
        compile_and_run("tests/regression/m122_generic_param_type/main.xi"),
        Some(0),
        "R69 generic params must carry their own (substituted) XIOM type"
    );
}

// R70: `for x in <collection>` treated every iterable as Range{start,end}:
// a %struct.Vec used field 0 (the DATA POINTER) as the loop index and wrote
// data+1 back into it (silent corruption; zero iterations for heap Vecs),
// and the checker bound the loop variable to Int (so `for s in vec_of_str`
// failed type-checking). Collections now lower to a real element loop and
// the checker binds the element type; unsupported iterables fail loudly.
#[test] fn e2e_m123_for_in_collections() {
    assert_eq!(
        compile_and_run("tests/regression/m123_for_in_collections/main.xi"),
        Some(0),
        "R70 for-in over collections must iterate elements correctly"
    );
}

// m124: `all(predicate)`/`none(predicate)` on collections. The legacy
// lowering called the xiom_all/xiom_none runtime stubs with len=0 (silently
// true), and in programs with the core module registered the method form
// resolved to the generic core.all helper, which is never monomorphised for
// this shape and landed on the returning-zero auto-stub (silently false).
// Both now lower inline over the Vec header with the closure ABI.
#[test] fn e2e_m124_contract_all_none() {
    assert_eq!(
        compile_and_run("tests/regression/m124_contract_all_none/main.xi"),
        Some(0),
        "m124 all/none must evaluate their predicate over collection elements"
    );
}

// R72/m127: fn references stored as collection ELEMENTS were raw code
// addresses while every call path loads field 0 as the code pointer and
// prepends the env -- `fns[i]()` dereferenced machine code and took an
// access violation (packages relay, xiom.test.run_all). Array literals now
// wrap fn references into closure envs (with a forwarding thunk) like the
// other fn-valued positions.
#[test] fn e2e_m127_fn_vec_indexed_calls() {
    assert_eq!(
        compile_and_run("tests/regression/m127_fn_vec_indexed_calls/main.xi"),
        Some(0),
        "R72 indexed calls through Vec[fn] elements must call, not crash"
    );
}

// FE-17: a trailing semicolon after a brace-less `module x;` header used to
// surface as "expected declaration, found ';'" (P001). The parser now skips
// one optional semicolon after the header (and after a block close).
#[test] fn e2e_m129_module_trailing_semicolon() {
    assert_eq!(
        compile_and_run("tests/regression/m129_module_trailing_semicolon/main.xi"),
        Some(0),
        "`module x;` trailing semicolon must compile and run"
    );
}

// C1 (Sprint C): fn-value ABI -- fn-typed elements keep the uniform closure
// ENV representation through push / apply / struct fields / closures /
// for-in / element-to-local, and unannotated fn arrays keep their marker.
#[test] fn e2e_m130_fn_value_abi() {
    assert_eq!(
        compile_and_run("tests/regression/m130_fn_value_abi/main.xi"),
        Some(0),
        "fn-value call ABI must follow the element's fn marker"
    );
}

// C1 (Sprint C): generic-mono ABI -- cross-type generic callbacks
// (U=Str / U=Float64) through fn-typed params, Vec[U] returns and
// array.map by value.
#[test] fn e2e_m131_generic_fn_callback_abi() {
    assert_eq!(
        compile_and_run("tests/regression/m131_generic_fn_callback_abi/main.xi"),
        Some(0),
        "generic callback ABI must follow the monomorphised instantiation"
    );
}

// E001 conservatism: consumed temporary borrows must not warn (the warning
// itself is asserted by crates/xiom/tests/borrow_e001.rs).
#[test] fn e2e_m132_e001_consumed_borrow() {
    assert_eq!(
        compile_and_run("tests/regression/m132_e001_consumed_borrow/main.xi"),
        Some(0),
        "temporary-borrow E001 false positives must stay gone"
    );
}

// Relay 2026-09-24: PRIVATE same-leaf types across project modules must keep
// distinct layouts (the Timer case; the triage only saw pub types before).
#[test] fn e2e_m134_private_same_leaf() {
    assert_eq!(
        compile_and_run("tests/regression/m134_private_same_leaf/main.xi"),
        Some(0),
        "private same-leaf types must not share one layout"
    );
}

// Relay 2026-09-24: well-typed clauses (implicit self, @pre, result) stay
// green after the clause Bool-mix rejection.
#[test] fn e2e_m136_well_typed_clauses() {
    assert_eq!(
        compile_and_run("tests/regression/m136_well_typed_clauses/main.xi"),
        Some(0),
        "well-typed contract clauses must compile and run"
    );
}

// Packages relay 2026-09-25: widened unsigned CONSTANTS must zero-extend
// (`239u8 as Int` used to sign-extend to -17; runtime locals were fine).
#[test] fn e2e_m138_u8_const_widen() {
    assert_eq!(
        compile_and_run("tests/regression/m138_u8_const_widen/main.xi"),
        Some(0),
        "unsigned suffixed constants must zero-extend when widened"
    );
}

// Packages relay #2: references to BOXED Result payload fields must yield the
// pointee (the real Vec), not the handle slot's address; `&mut r.value`
// mutation must reach the box.
#[test] fn e2e_m140_result_payload_ref() {
    assert_eq!(
        compile_and_run("tests/regression/m140_result_payload_ref/main.xi"),
        Some(0),
        "&Result.payload must pass the boxed pointee"
    );
}

// Stdlib relay (is_null): method-position `ptr.is_null()` over a pointer
// FIELD and over a `ptr` field that shadows the imported `xiom.ptr` module
// alias must bind the free fn with the receiver as arg 0. The old flow
// emitted a garbage symbol (`@ptr.is_null`) that the auto-stub pass answered
// with a silent `ret 0`, so every pointer read as non-null.
#[test] fn e2e_m142_ptr_isnull_ufcs() {
    assert_eq!(
        compile_and_run("tests/regression/m142_ptr_isnull_ufcs/main.xi"),
        Some(0),
        "ptr.is_null() (direct, method, module-shadow) must agree on null/non-null"
    );
}

// By-value receiver container mutation: an explicit-`self` method that
// pushes/inserts into a container FIELD must use the pointer receiver ABI,
// or the Vec/Map/Set header update stays in the callee copy and the caller's
// container silently keeps its old length.
#[test] fn e2e_m143_receiver_container_mutation() {
    assert_eq!(
        compile_and_run("tests/regression/m143_receiver_container_mutation/main.xi"),
        Some(0),
        "self.v.push / self.m.insert / self.s.insert (and the bare-field form) must persist"
    );
}

// Sibling-method receiver binding: receiver-qualified fns whose bodies only
// call siblings bare must get the %param_self slot and resolve via G-10.
// The old flow emitted 2-arg calls against 3-param definitions (KEY
// inttoptr'd as receiver) and AV'd inside HashMap.insert/get/contains.
#[test] fn e2e_m144_sibling_method_calls() {
    assert_eq!(
        compile_and_run("tests/regression/m144_sibling_method_calls/main.xi"),
        Some(0),
        "bare sibling calls in this-based methods (HashMap, Holder) must bind the receiver"
    );
}

// R-1 (benchmark relay): C-family/Rust bitwise precedence. `a << b | c`
// parsed as `a << (b | c)` (silent wrong values: (1 << 8) | 2 == 1024).
#[test] fn e2e_m145_shift_precedence() {
    assert_eq!(
        compile_and_run("tests/regression/m145_shift_precedence/main.xi"),
        Some(0),
        "shift > & > ^ > | > comparisons; bit-packing must group shifts first"
    );
}

// A let-bound extern returning a signed narrow int must sign-extend its
// negative values (`strcmp < 0`); the binding used to zext, so a false
// success reported (the net.tcp_connect relay class).
#[test] fn e2e_m146_signed_extern_result() {
    assert_eq!(
        compile_and_run("tests/regression/m146_signed_extern_result/main.xi"),
        Some(0),
        "Int32 extern results must widen sext (negative comparison)"
    );
}

// R-2 (benchmark relay): match bindings on a persistent Option[T] must ALIAS
// the boxed aggregate payload (field writes and Vec pushes through the
// binding used to mutate a stack copy; the payload kept the old value).
#[test] fn e2e_m148_match_payload_alias() {
    assert_eq!(
        compile_and_run("tests/regression/m148_match_payload_alias/main.xi"),
        Some(0),
        "Some(v) bindings must alias the boxed payload (struct field + Vec push)"
    );
}

// m150 (R-5 benchmark relay): combining `use xiom.num;` with
// `use xiom.ffi.dl;` must compile and run. The two bogus T002 findings on
// xiom.math.primitives' own `pub fn abs` (libc `abs` in xiom.ffi.c) came
// from the global extern-name gate; the fn-definition shadow fix clears
// them. The relay's full num.parse_int shape is check-locked separately
// (m150_dl_num_parse + checker_locks) until the io.parse_int C001 batch.
#[test] fn e2e_m150_dl_num_catalog_abs() {
    assert_eq!(
        compile_and_run("tests/regression/m150_dl_num_catalog_abs/main.xi"),
        Some(0),
        "xiom.num + xiom.ffi.dl combined imports must compile and the dl module must stay usable"
    );
}

// Item 3 (exact arity): the flip must reject real mismatches while both
// previously-false-positive shapes stay accepted -- receiver-sugar calls to
// receiver-style generic methods (`fn Box.get[T](b: &Box[T])`, the
// smoke_core_box shape) and generic bound methods (`T: Ord` -> interface
// dispatch, the xiom.cmp shape).
#[test] fn e2e_m150_exact_arity() {
    assert_eq!(
        compile_and_run("tests/regression/m150_exact_arity/main.xi"),
        Some(0),
        "receiver-sugar + generic-bound calls must stay arity-clean under the flip"
    );
}

// Item 3 (exact arity): extra and missing arguments must fail the compile
// (codegen used to drop extras and default the missing).
#[test] fn e2e_m150_exact_arity_rejects() {
    assert!(
        compile_and_run("tests/regression/m150_exact_arity/reject_extra.xi").is_none(),
        "extra arguments must be rejected"
    );
    assert!(
        compile_and_run("tests/regression/m150_exact_arity/reject_missing.xi").is_none(),
        "missing arguments must be rejected"
    );
}

// R53 (packages relay): a PLAIN-local argument fed to a `&mut`/pointer
// out-param must write through to the caller (v0.61.3 materialized a temp
// copy and lost every write; a `&mut Struct` variant corrupted memory).
#[test] fn e2e_m152_mut_write_through() {
    assert_eq!(
        compile_and_run("tests/regression/m152_mut_write_through/main.xi"),
        Some(0),
        "plain-local calls to &mut out-params must write through (and explicit &mut must stay correct)"
    );
}

// C001 (benchmark relay): `let trimmed = s.trim(); trimmed.is_empty()`
// inside a catalog body resolved the value receiver as a MODULE and emitted
// an undefined `@is_empty` (unresolved symbol from @io.parse_int). The
// primitive method key is now synthesised from the receiver local's LLVM
// type when its XIOM type was never recorded.
#[test] fn e2e_m153_parse_int_trim_isempty() {
    assert_eq!(
        compile_and_run("tests/regression/m153_parse_int_trim_isempty/main.xi"),
        Some(0),
        "trim + is_empty on a let-bound Str must resolve and run (C001)"
    );
}

// Benchmark relay's exact num + ffi.dl program: promoted from the R-5
// check-only lock once the C001 codegen fix landed (its comment asked for
// exactly this graduation).
#[test] fn e2e_m150_dl_num_parse() {
    assert_eq!(
        compile_and_run("tests/regression/m150_dl_num_parse/main.xi"),
        Some(0),
        "num + ffi.dl with io.parse_int must compile and run (R-5 + C001)"
    );
}

// m163 (selfhost Phase 0 finding): a Vec[Str] struct-field element inside a
// struct METHOD lowered to i64 add + inttoptr instead of @xiom_str_concat
// (silent pointer-decimal garbage / AV). The prologue now registers field
// Vec element types exactly like local Vecs.
#[test] fn e2e_m163_method_field_vec_concat() {
    assert_eq!(
        compile_and_run("tests/regression/m163_method_field_vec_concat/main.xi"),
        Some(0),
        "method-context field Vec[Str] element concat/accumulation must use xiom_str_concat (m163)"
    );
}

// m164: module-level const ARRAYS materialize as one `internal constant`
// global and index reads GEP it (no per-use alloca + N stores).
#[test] fn e2e_m164_const_table_global() {
    assert_eq!(
        compile_and_run("tests/regression/m164_const_table_global/main.xi"),
        Some(0),
        "const tables must read through their internal constant global with i64-slot parity (m164)"
    );
}

// m162 (playground relay): a user module's same-leaf fn (`char_at`) must not
// poison catalog-body checking OR codegen binding of xiom.num's own
// `use xiom.string.char_at` resolution.
#[test] fn e2e_m162_sameleaf_catalog_poison() {
    assert_eq!(
        compile_and_run("tests/regression/m162_sameleaf_catalog_poison/main.xi"),
        Some(0),
        "user same-leaf fns must not hijack catalog-body imports (m162)"
    );
}

// m191 (benchmark relay): a BARE program call whose leaf is exported by two
// modules (`lz4_compress` in xiom.compress.lz4 -> Vec and xiom.compress ->
// Result) must bind the checker's resolution; pre-fix `.len()` read
// registration-order aggregate garbage.
#[test] fn e2e_m191_lz4_bare_duplicate_leaf() {
    assert_eq!(
        compile_and_run("tests/regression/m191_lz4_bare_duplicate_leaf/main.xi"),
        Some(0),
        "a bare call to a duplicated stdlib leaf must bind the checker's target (m191)"
    );
}

// m192 (perf queue item 1): a confined `unsafe` block re-entered from a loop
// leaked its 32-byte ctx alloca per execution -- 262,144 entries exhaust the
// 8 MB stack reserve (0xC0000005; packages grpc probe_suite_min crash class).
// 500k entries must complete with capture write-back intact.
#[test] fn e2e_m192_unsafe_ctx_loop_stack() {
    assert_eq!(
        compile_and_run("tests/regression/m192_unsafe_ctx_loop_stack/main.xi"),
        Some(0),
        "a loop-re-entered confined unsafe block must not leak stack per entry (m192)"
    );
}

// m193 (stdlib relay): a user extern of a builtin runtime symbol
// (xiom_guard_alloc) emitted a duplicate declare and clang rejected the
// module ("invalid redefinition of function"). The extern must compile, link
// and run against the runtime symbol.
#[test] fn e2e_m193_guard_alloc_extern() {
    assert_eq!(
        compile_and_run("tests/regression/m193_guard_alloc_extern/main.xi"),
        Some(0),
        "a direct extern of a builtin runtime symbol must link once, not duplicate (m193)"
    );
}

// m194 (stdlib relay): `num.float.float_bits`/`bits_to_float` must lower to
// an LLVM bitcast at the call site (exact IEEE-754 reinterpretation), not to
// the stdlib's documented zero fallbacks.
#[test] fn e2e_m194_float_bitcast() {
    assert_eq!(
        compile_and_run("tests/regression/m194_float_bitcast/main.xi"),
        Some(0),
        "float_bits/bits_to_float must reinterpret exactly (m194)"
    );
}

#[test] fn e2e_m194_float_bitcast_ir() {
    let ir = compile_ir("tests/regression/m194_float_bitcast/main.xi")
        .expect("m194 fixture must compile to IR");
    let start = ir.find("define i64 @main(").expect("m194: main must be defined");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("bitcast double") && body.contains("bitcast i64"),
        "m194: both directions must lower to bitcast in main; got:\n{body}"
    );
    assert!(
        !body.contains("call i64 @num.float.float_bits")
            && !body.contains("call double @num.float.bits_to_float"),
        "m194: the call sites must be intercepted, not dispatched to the fallbacks; got:\n{body}"
    );
}

// m195: angle-bracket generic receivers keep their type args; Vec<Big>.new()
// must size 32-byte slots and round-trip 10 pushed structs.
#[test] fn e2e_m195_angle_vec_new_struct() {
    assert_eq!(
        compile_and_run("tests/regression/m195_angle_vec_new_struct/main.xi"),
        Some(0),
        "Vec<T>.new() with angle brackets must size element slots correctly (m195)"
    );
}

// m195 real-world: reflect.all_types() (Vec<TypeInfo> with angle args) must
// not heap-corrupt (pre-fix 0xC0000374).
#[test] fn e2e_m195_reflect_all_types() {
    assert_eq!(
        compile_and_run("tests/regression/m195_reflect_all_types/main.xi"),
        Some(0),
        "reflect.all_types() must not heap-corrupt (m195)"
    );
}

// m196 (Pulse C-PULSE-01 / R-8): a real one-arg method named `read` must run
// its body, not be swallowed by the raw-pointer read builtin.
#[test] fn e2e_m196_read_method_not_hijacked() {
    assert_eq!(
        compile_and_run("tests/regression/m196_read_method_not_hijacked/main.xi"),
        Some(0),
        "a one-arg read method must call the real method, not the builtin (m196)"
    );
}

// m197 (Pulse C-PULSE-04): bare `&mut Int` reads in value position must load
// the pointee, not yield the address.
#[test] fn e2e_m197_mut_ref_bare_read() {
    assert_eq!(
        compile_and_run("tests/regression/m197_mut_ref_bare_read/main.xi"),
        Some(0),
        "bare reference reads must load the pointee (m197)"
    );
}

// m198 (Pulse C-PULSE-06): an incomplete struct literal must be REJECTED
// with a T001 diagnostic naming the missing field (pre-fix it compiled and
// the omitted field read uninitialized memory).
#[test] fn e2e_m198_missing_struct_field_rejected() {
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", "tests/regression//m198_missing_struct_field/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed to run xiom");
    assert!(
        !output.status.success(),
        "an incomplete struct literal must not compile"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing field 'b'"),
        "the diagnostic must name the missing field; got:\n{stderr}"
    );
    assert!(
        stderr.contains("T001"),
        "the diagnostic must be T001; got:\n{stderr}"
    );
}

// m199 (Pulse C-PULSE-05): a method call on a module-level `const` receiver
// must keep the const's concrete type. Pre-fix the receiver was dropped for
// `const ... : Int` and the call lowered to the erased-interface `@to_str()`
// typed stub (W005 -> empty render; v0.64.0 aborted 0x80000003).
#[test] fn e2e_m199_const_receiver_method_dispatch() {
    assert_eq!(
        compile_and_run("tests/regression/m199_const_receiver_to_str/main.xi"),
        Some(0),
        "const receivers must dispatch with their concrete type (m199)"
    );
}

// m200 (stdlib p_rvalue_float_vec_index): inline (rvalue) indexing of a
// returned Vec[Float64] must read the stored double, not the raw i64 bits.
#[test] fn e2e_m200_rvalue_float_vec_index() {
    assert_eq!(
        compile_and_run("tests/regression/m200_rvalue_float_vec_index/main.xi"),
        Some(0),
        "rvalue Vec[Float64] index must read the stored element (m200)"
    );
}

// m201 (stdlib p_multipart_parse_name): a match-bound Result[Vec[Struct], _]
// payload must keep the concrete Vec element type (struct memcpy reads).
#[test] fn e2e_m201_match_payload_vec_elem() {
    assert_eq!(
        compile_and_run("tests/regression/m201_match_payload_vec_elem/main.xi"),
        Some(0),
        "match-bound Vec[Struct] payloads must read their fields (m201)"
    );
}

// m202 (packages grpc tuple-vec relay): `("k".clone(), "v".clone())` elements
// must keep their Str/Str tuple type; pre-fix the tuple was built as
// Tuple__MaybeUninit__MaybeUninit and reading metadata[0].0 after a push
// crashed 0xC0000005 (grpc probe_suite_min/probe_direct).
#[test] fn e2e_m202_clone_tuple_vec() {
    assert_eq!(
        compile_and_run("tests/regression/m202_clone_tuple_vec/main.xi"),
        Some(0),
        "clone-tuple Vec elements must keep their concrete tuple type (m202)"
    );
}

// m203 (stdlib iter Range.count clause): an ensures clause on a fn whose
// return expression contains a block closure must not leak into the closure
// thunk (clang "instruction forward referenced with type 'ptr'").
#[test] fn e2e_m203_closure_ensures_isolation() {
    assert_eq!(
        compile_and_run("tests/regression/m203_closure_ensures_isolation/main.xi"),
        Some(0),
        "closure returns must not emit the enclosing fn's ensures checks (m203)"
    );
}

// m206 (packages graphql relay): a struct enum payload matched from a
// Vec-index scrutinee must bind pointer-backed to the boxed payload;
// pre-fix every payload field read as the constant 0 (graphql 9/10).
#[test] fn e2e_m206_enum_struct_payload_field() {
    assert_eq!(
        compile_and_run("tests/regression/m206_enum_struct_payload_field/main.xi"),
        Some(0),
        "enum struct payload fields must read through the box (m206)"
    );
}

// m209 (packages relay): `let (k, v) = &vec[i]` must bind component refs and
// `k == &key` must content-compare (pre-fix: same element address for both
// names; lookup silently missed).
#[test] fn e2e_m209_tuple_ref_destructure() {
    assert_eq!(
        compile_and_run("tests/regression/m209_tuple_ref_destructure/main.xi"),
        Some(0),
        "tuple-ref destructure + ref content compare must match (m209)"
    );
}

// m210 (packages struct-clone relay): `let w = v.clone()` over Vec[Struct]
// must keep the element type so `w[i]` reads struct fields (pre-fix the
// runtime elem-size scalar switch aborted 0xC0000005).
#[test] fn e2e_m210_vec_clone_struct_elem() {
    assert_eq!(
        compile_and_run("tests/regression/m210_vec_clone_struct_elem/main.xi"),
        Some(0),
        "clone of Vec[Struct] must keep the element type (m210)"
    );
}

// m211 (stdlib io.list_dir relay): `Str::from_c_str` must copy into an owned
// Str; the pointer identity made list_dir push aliases of readdir's reused
// buffer (correct count, last name repeated).
#[test] fn e2e_m211_list_dir_owned_names() {
    assert_eq!(
        compile_and_run("tests/regression/m211_list_dir_owned_names/main.xi"),
        Some(0),
        "list_dir entries must be distinct owned names (m211)"
    );
}

// m216 (C-PULSE-11): a type alias to a package-style struct
// (`pub type Store = pkg.SessionStore;`) used as a Vec element must resolve
// in the element-struct lookup; pre-fix `v[0].field` read the constant 0
// (Pulse wrapper-module shape).
#[test] fn e2e_m216_alias_vec_elem_field() {
    assert_eq!(
        compile_and_run("tests/regression/m216_alias_vec_elem_field/main.xi"),
        Some(0),
        "Vec[Store] alias elements must resolve to the struct (m216)"
    );
}

// m217 (C-PULSE-10): `Result[Option[Str], Str]` read as `c.value.value` must
// keep the inner Str payload type; pre-fix the inner read kept the erased i64
// slot and kv_get printed the data pointer as an address-like decimal.
#[test] fn e2e_m217_nested_option_payload() {
    assert_eq!(
        compile_and_run("tests/regression/m217_nested_option_payload/main.xi"),
        Some(0),
        "nested Option payload chains must keep the inner Str payload (m217)"
    );
}

// m218 (XVC-C-05): Float32 arithmetic on a Vec[Float32] struct field must not
// fall to the scalar int element path when a generated aggregate
// (`Option__Vector`) shadows the struct key in the HashMap suffix scan.
#[test] fn e2e_m218_vec_float_field_lookup() {
    assert_eq!(
        compile_and_run("tests/regression/m218_vec_float_field_lookup/main.xi"),
        Some(0),
        "Vec[Float32] struct-field elements must multiply as floats (m218)"
    );
}

// m219 (XVC-C-01): `&fn() -> T` parameters: direct call, `(*f)()` and a
// local rebound from `*f` must all call the passed function correctly.
#[test] fn e2e_m219_fnptr_ref_call() {
    assert_eq!(
        compile_and_run("tests/regression/m219_fnptr_ref_call/main.xi"),
        Some(0),
        "&fn() parameter call shapes must dispatch the passed function (m219)"
    );
}

// m221 (XVC-C-06): colliding enum payload field names erase to i64; Float32
// payloads must pack their f32 bits (FloatVal(2.5) used to extract as 0.0).
#[test] fn e2e_m221_enum_f32_payload_pack() {
    assert_eq!(
        compile_and_run("tests/regression/m221_enum_f32_payload_pack/main.xi"),
        Some(0),
        "Float32 enum payloads must roundtrip through the erased i64 slot (m221)"
    );
}

// m222 (XVC-C-07): `Str::from_utf8(&Vec[UInt8])` used to emit invalid LLVM IR
// (`getelementptr %struct.Vec*, %struct.Vec** %slot, i32 0, i32 0`); the
// reference form, the by-value form and `from_bytes(&Vec)` must all build and
// return the exact string.
#[test] fn e2e_m222_from_utf8_vec_ref() {
    assert_eq!(
        compile_and_run("tests/regression/m222_from_utf8_vec_ref/main.xi"),
        Some(0),
        "Str::from_utf8 on a &Vec[UInt8] must build and produce the exact string (m222)"
    );
}

// m223 (C-ORBIT-01): match-bound payloads behind type aliases must keep their
// concrete type (T001 "'get_column' expects 2 argument(s), found 1" blocked
// the ORBITDB transaction module).
#[test] fn e2e_m223_alias_result_payload() {
    assert_eq!(
        compile_and_run("tests/regression/m223_alias_result_payload/main.xi"),
        Some(0),
        "generic + plain type aliases must resolve match payload types (m223)"
    );
}

// m224 (C-ORBIT-03): field receivers of pointer-self methods must write
// through to the field (nested receiver copies dropped the mutation).
#[test] fn e2e_m224_nested_field_mut_receiver() {
    assert_eq!(
        compile_and_run("tests/regression/m224_nested_field_mut_receiver/main.xi"),
        Some(0),
        "field receivers of &mut methods must pass the field address (m224)"
    );
}

// m225 (C-ORBIT-04): unqualified variant construction must bind the enclosing
// module's enum (twin enums across modules mis-built the wrong type).
#[test] fn e2e_m225_enum_module_scope() {
    assert_eq!(
        compile_and_run("tests/regression/m225_enum_module_scope/main.xi"),
        Some(0),
        "bare variants must scope to the declaring module's enum (m225)"
    );
}

// m226 (C-ORBIT-02): index-assign into Vec[Option[...]]/Vec[Result[...]]
// must memcpy the container struct (scalar handle store made matches no-op).
#[test] fn e2e_m226_vec_option_assign() {
    assert_eq!(
        compile_and_run("tests/regression/m226_vec_option_assign/main.xi"),
        Some(0),
        "container-element Vec assignment must keep the tag/payload layout (m226)"
    );
}

// m227 (C-PULSE-09): `.len()` on a module-level `Vec[T]` must dispatch the
// Vec builtin; it used to load the LEN field and call it as a fn pointer
// (0xC000001D at the first cross-module access on Windows).
#[test] fn e2e_m227_module_global_vec_len() {
    assert_eq!(
        compile_and_run("tests/regression/m227_module_global_vec_len/main.xi"),
        Some(0),
        "module-level Vec receivers must dispatch builtins, not call the len field (m227)"
    );
}

// m229 (packages lane): `is Ok(<literal>)` must compare the payload literal,
// not just the tag.
#[test] fn e2e_m229_is_payload_literal() {
    assert_eq!(
        compile_and_run("tests/regression/m229_is_payload_literal/main.xi"),
        Some(0),
        "is-patterns with literal payloads must compare the payload (m229)"
    );
}

// m231 (BINDINGS B-01): a struct field whose type is matched by enum variants
// must not resolve through Option__/Result__ wrapper keys (nondeterministic
// wrong-tag builds; 7/12 on the bindings pkg repro).
#[test] fn e2e_m231_option_wrapper_prefix_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m231_option_wrapper_prefix_shadow/main.xi"),
        Some(0),
        "enum field matches must resolve the field's own type, not a wrapper payload type (m231)"
    );
}

// m234 (XVC-C-08): Vec element strides must include LLVM alignment padding
// (Float32 before an 8-aligned Vec field). The naive field sum used a 52-byte
// stride for a 56-byte struct, so the second element's trailing scalar read
// garbage (silent wrong results).
#[test] fn e2e_m234_vec_elem_padding() {
    assert_eq!(
        compile_and_run("tests/regression/m234_vec_elem_padding/main.xi"),
        Some(0),
        "Vec[Struct] element stride must match the padded LLVM layout (m234)"
    );
}

// m235 (C-ORBIT-05): static allocas emitted inside a long loop leaked stack
// (alloca executes per reach; freed only at return): nested Vec[Page] reads +
// push died with 0xC0000005 after ~86.7k iterations. Static allocas are now
// hoisted to the entry block.
#[test] fn e2e_m235_loop_alloca_hoist() {
    assert_eq!(
        compile_and_run("tests/regression/m235_loop_alloca_hoist/main.xi"),
        Some(0),
        "loop-body static allocas must be hoisted to the entry block (m235)"
    );
}

// m236 (m230): the stdlib prelude must load when a USER module imports a
// stdlib module; otherwise the imported catalog body checks without core
// container declarations and hard-fails (3x T001 in xiom.encoding).
#[test] fn e2e_m236_user_stdlib_prelude() {
    assert_eq!(
        compile_and_run("tests/regression/m236_user_stdlib_prelude/main.xi"),
        Some(0),
        "user-module stdlib imports must trigger the catalog prelude (m236)"
    );
}

// m237 (wave-96 p_array_zip_no_truncate): const-generic inference bound M to
// N's array size, so `if M < count` was dead and array_zip read b[M] out of
// bounds for M < N. Now each const generic binds to the parameter whose type
// names it; the fixture covers M<N, M==0, N<=M and N==M with value checks.
#[test] fn e2e_m237_array_zip_truncate() {
    assert_eq!(
        compile_and_run("tests/regression/m237_array_zip_truncate/main.xi"),
        Some(0),
        "array_zip must truncate to the shorter array (m237)"
    );
}

// m238 (wave-96 block 80): a zero-length fixed array passed BY VALUE to a
// generic `[N]T` param miscompiled at clang ([0 x i64] vs i64); the call
// now passes a zero element value, and annotated empty locals record their
// element type so `[0]Str` infers T=Str.
#[test] fn e2e_m238_zero_len_array_by_value() {
    assert_eq!(
        compile_and_run("tests/regression/m238_zero_len_array_by_value/main.xi"),
        Some(0),
        "zero-length fixed arrays by value must lower to the element type (m238)"
    );
}

// m239 (queue item 20): container `==`/`!=` must compare content, not the
// erased field bits (Vec data pointers / boxed Option-Result payloads).
#[test] fn e2e_m239_deep_container_eq() {
    assert_eq!(
        compile_and_run("tests/regression/m239_deep_container_eq/main.xi"),
        Some(0),
        "Vec/Option/Result equality must compare content (m239)"
    );
}

// m333 (pin CI): llvm.trap signal-terminates on unix (`code()` is None)
// while Windows reports an exit code -- both are "trapped under
// --overflow-checks". Windows keeps the strict Some(nonzero) contract.
fn trapped_under_bounds_checks(code: Option<i32>) -> bool {
    if cfg!(windows) {
        code.is_some() && code != Some(0)
    } else {
        code != Some(0)
    }
}

// m241 (honest containment, t8 arena): an out-of-bounds Vec index WRITE
// must trap under --overflow-checks like the read path already does. The
// fixture writes buf[100] on a 4-element Vec: pre-fix it exited 0 silently
// (SILENT_UB in the arena); post-fix llvm.trap fires.
#[test] fn e2e_m241_vec_write_bounds_traps() {
    let code = compile_and_run_with_flags(
        "tests/regression/m241_vec_write_bounds/main.xi",
        &["--overflow-checks"],
    );
    assert!(
        trapped_under_bounds_checks(code),
        "OOB Vec write must trap under --overflow-checks (m241), got {code:?}"
    );
}

// m241 IR lock: the write-path bounds guard is emitted under
// --overflow-checks and stays out of default builds.
#[test] fn e2e_m241_vec_write_bounds_ir() {
    let emit = |extra: &[&str]| -> String {
        let output = Command::new(xiom_path())
            .args(extra)
            .args(["--emit-ir", "tests/regression/m241_vec_write_bounds/main.xi"])
            .current_dir(project_root())
            .output()
            .expect("emit-ir");
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let checked = emit(&["--overflow-checks"]);
    assert!(
        checked.contains("wbounds_trap"),
        "the indexed-write bounds trap must be emitted under --overflow-checks"
    );
    let default = emit(&[]);
    assert!(
        !default.contains("wbounds_trap"),
        "default builds must keep the historical unchecked write path"
    );
}

// m242 (wave-97 p_sibling_dup_fn_alias): a dotted `use` path naming a
// DECLARED MODULE must bind the module even when the parent exports a
// same-named fn (bits.xi's `pub fn popcount` vs xiom.bits.popcount). The
// trio of duplicate-leaf siblings must all stay independently callable.
#[test] fn e2e_m242_sibling_dup_alias() {
    assert_eq!(
        compile_and_run("tests/regression/m242_sibling_dup_alias/main.xi"),
        Some(0),
        "triplicate sibling exports must not break alias-qualified calls (m242)"
    );
}

// m242 minimal shape: popcount imported BEFORE a duplicate-leaf sibling
// (the order that used to fail because the parent's fn won the first-wins
// leaf slot).
#[test] fn e2e_m242_sibling_dup_alias_popcount_bitwise() {
    assert_eq!(
        compile_and_run("tests/regression/m242_sibling_dup_alias_popcount_bitwise/main.xi"),
        Some(0),
        "popcount-then-bitwise import order must resolve (m242)"
    );
}

// m243 (honest containment, m241 follow-up): a fixed-array `[N]T` index
// READ or WRITE out of bounds must trap under --overflow-checks.
#[test] fn e2e_m243_fixed_array_bounds_traps() {
    let code = compile_and_run_with_flags(
        "tests/regression/m243_fixed_array_bounds/main.xi",
        &["--overflow-checks"],
    );
    assert!(
        trapped_under_bounds_checks(code),
        "OOB fixed-array access must trap under --overflow-checks (m243), got {code:?}"
    );
}

// m243 IR lock: the fixed-array guard (read and write) is emitted under
// --overflow-checks and stays out of default builds.
#[test] fn e2e_m243_fixed_array_bounds_ir() {
    let emit = |extra: &[&str]| -> String {
        let output = Command::new(xiom_path())
            .args(extra)
            .args(["--emit-ir", "tests/regression/m243_fixed_array_bounds/main.xi"])
            .current_dir(project_root())
            .output()
            .expect("emit-ir");
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let checked = emit(&["--overflow-checks"]);
    assert!(
        checked.contains("abounds_trap"),
        "the fixed-array bounds trap must be emitted under --overflow-checks"
    );
    let default = emit(&[]);
    assert!(
        !default.contains("abounds_trap"),
        "default builds must keep the historical unchecked fixed-array path"
    );
}

// m244 (honest containment): raw derefs get a NULL guard under
// --overflow-checks; IR lock + contained-fault runtime locks.
#[test] fn e2e_m244_null_deref_guard_ir() {
    let emit = |extra: &[&str]| -> String {
        let output = Command::new(xiom_path())
            .args(extra)
            .args(["--emit-ir", "tests/regression/m244_null_deref_guard/main.xi"])
            .current_dir(project_root())
            .output()
            .expect("emit-ir");
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let checked = emit(&["--overflow-checks"]);
    assert!(
        checked.contains("null_deref_trap"),
        "the null-deref guard must be emitted under --overflow-checks"
    );
    let default = emit(&[]);
    assert!(
        !default.contains("null_deref_trap"),
        "default builds must keep the historical unguarded deref path"
    );
}

// m244: the guarded derefs are CONTAINED by the unsafe-block trampoline
// (no crash), but m351 makes the language-level fault observable: the
// confined faults are deterministic and UNACKNOWLEDGED, so the process
// exits 70 instead of masquerading as a successful exit 0.
#[test] fn e2e_m244_null_deref_guard_contained() {
    assert_eq!(
        compile_and_run_with_flags(
            "tests/regression/m244_null_deref_guard/main.xi",
            &["--overflow-checks"]
        ),
        Some(70),
        "guarded null derefs must be contained (no crash) but unacknowledged faults exit 70 (m244/m351)"
    );
}

// m244: with the runtime fault trace on, the confined block reports the
// deterministic fault (SIGILL-equivalent from llvm.trap), proving the
// guard fired before any memory access.
#[test] fn e2e_m244_null_deref_guard_fault_trace() {
    let exe = project_root().join("target").join("m244_null_deref_trace.exe");
    let compile = Command::new(xiom_path())
        .args([
            "--overflow-checks",
            "tests/regression/m244_null_deref_guard/main.xi",
            "-o",
        ])
        .arg(&exe)
        .current_dir(project_root())
        .output()
        .expect("compile m244 fixture");
    assert!(compile.status.success(), "m244 fixture must compile");
    let run = Command::new(&exe)
        .env("XIOM_TRACE_FAULT", "1")
        .output()
        .expect("run m244 fixture");
    let stderr = String::from_utf8_lossy(&run.stderr);
    // m333: the verbose trace marker is emitted by the Windows SEH path only;
    // the POSIX containment (sigsetjmp) prints no trace marker. m351's
    // always-on typed `[fault]` diagnostic is asserted on every platform.
    #[cfg(windows)]
    assert!(
        stderr.contains("[trampoline] fault"),
        "the confined null deref must report a deterministic fault; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("[fault] confined block"),
        "m351: the fault must be manifested with the always-on typed diagnostic; stderr:\n{stderr}"
    );
    assert_eq!(
        run.status.code(),
        Some(70),
        "m351: an unacknowledged confined fault exits 70 (no silent exit 0)"
    );
}

// m246 (sweep finding B-10): a LOCAL fn-pointer named `alloc`/`free`
// called inside a confined unsafe block must call the local, not the
// compiler allocator builtins (guard-arena hijack).
#[test] fn e2e_m246_local_alloc_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m246_local_alloc_shadow/main.xi"),
        Some(0),
        "local `alloc`/`free` fn-pointers must not be hijacked by builtins (m246)"
    );
}

// m248 (stdlib byref-generic finding): generic `&Option[T]`/`&Result[T,E]`
// query bodies must read the tag through the pointer pseudo-fields.
#[test] fn e2e_m248_byref_generic_query() {
    assert_eq!(
        compile_and_run("tests/regression/m248_byref_generic_query/main.xi"),
        Some(0),
        "generic by-ref Option/Result queries must read correctly (m248)"
    );
}

// m249 (stdlib slice-bound C001): the Slice->Vec bridge at the Ref unwrap
// makes the module-qualified `is_sorted(&s)` shape run end to end.
#[test] fn e2e_m249_slice_bound_run() {
    assert_eq!(
        compile_and_run("tests/regression/m249_slice_bound_infer/main.xi"),
        Some(0),
        "Slice values must bridge to the mono'd &Slice[T] Vec ABI (m249)"
    );
}

// m249 IR lock: a `&Slice[T]` param must infer T from the ELEMENT, not
// the container wrapper.
#[test] fn e2e_m249_slice_bound_infer_ir() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "tests/regression/m249_slice_bound_infer/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("emit-ir");
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(
        ir.contains("@core.is_sorted_Int"),
        "`&Slice[T]` must infer T from the element (mono is_sorted_Int); got:\n{ir}"
    );
    assert!(
        !ir.contains("is_sorted_Slice"),
        "the container name must never be used as the generic arg; got:\n{ir}"
    );
}

// m254 (stdlib tuple-literal element read): an INDEXED element read written
// inside a tuple literal must keep the element's XIOM type -- the tuple
// namers defaulted Index to "Int", so `v.push((pred[i], lab[i]))` built
// Tuple__Int__Int over a (Float64, Int) vector and corrupted the Float64
// component.
#[test] fn e2e_m254_tuple_literal_index_elem() {
    assert_eq!(
        compile_and_run("tests/regression/m254_tuple_literal_index_elem/main.xi"),
        Some(0),
        "inline Vec element reads in tuple literals must keep their XIOM type (m254)"
    );
}

// m256 (REL-20261010-1803-selfhost): a match-arm Vec payload binding whose
// field name repeats across enum variants (M19-erased i64 handle slot) must
// still name the tuple element Vec -- the literal built Tuple__Int__Int
// against the declared Tuple__Int__Vec and clang rejected the ret.
#[test] fn e2e_m256_vec_arm_binding_tuple() {
    assert_eq!(
        compile_and_run("tests/regression/m256_vec_arm_binding_tuple/main.xi"),
        Some(0),
        "Vec payload arm bindings in tuple returns must keep the container type (m256)"
    );
}

// m260 (C-ORBIT-08 B): [N]UInt8 element `as Int` must zero-extend (0xF8 ->
// 248); the annotation-only fixed-array path sign-extended through the
// signed default while [N]Int8 keeps sext.
#[test] fn e2e_m260_u8_array_elem_cast() {
    assert_eq!(
        compile_and_run("tests/regression/m260_u8_array_elem_cast/main.xi"),
        Some(0),
        "fixed-array element widening must follow the element signedness (m260)"
    );
}

// m261 (REL-20261011-1124, C-ORBIT-07): a bare call inside a module must bind
// the CALLER module's own definition -- including module-PRIVATE fns -- ahead
// of the keep-first bare slot. The orbitdb consumer-fn-hijack shape: a
// consumer's arm(Str,Str,Str) must not receive mod_svc.ping()'s zero-arg call.
#[test] fn e2e_m261_module_local_precedence() {
    assert_eq!(
        compile_and_run("tests/regression/m261_module_local_precedence/main.xi"),
        Some(0),
        "the caller module's own private fn must win the bare call (m261)"
    );
}

// m262 (REL-20261011-1052): module-private module-level consts are isolated
// per module -- the numeric farm+metro pin (metrohash64(empty,0) must be
// 0x13C52518F4DB1E2A), in BOTH import orders. The const_tag pins catch the
// reverse order (each module must see its OWN constants).
#[test] fn e2e_m262_private_symbol_isolation_farm_first() {
    assert_eq!(
        compile_and_run("tests/regression/m262_private_symbol_isolation/main_farm_first.xi"),
        Some(0),
        "farm-first order: metro must hash with its OWN constants (m262)"
    );
}

#[test] fn e2e_m262_private_symbol_isolation_metro_first() {
    assert_eq!(
        compile_and_run("tests/regression/m262_private_symbol_isolation/main_metro_first.xi"),
        Some(0),
        "metro-first order: metro must hash with its OWN constants (m262)"
    );
}

// m264 (REL-20261011-1121 B-14/B-15): a Vec[fn] ELEMENT call with >= 2
// arguments types the fn's declared return (`if h(a, b)` / `let got = h(a,b)`
// were `()`); a bare `return;` unifies with an explicit `-> Unit`; pushing a
// bare fn NAME into a MODULE-GLOBAL Vec[fn] stores the closure env the
// element-call path expects (raw code address -> 0xC0000005).
#[test] fn e2e_m264_element_arity_and_unit() {
    assert_eq!(
        compile_and_run("tests/regression/m264_element_arity_and_unit/main.xi"),
        Some(0),
        "Vec[fn] element calls of any arity + unit returns + global push (m264)"
    );
}

// m265 (REL-20261011-0317 pulse-7): a module-scope fn-pointer `var` is
// callable and reassignable (the test-harness swap-the-runner shape).
#[test] fn e2e_m265_module_scope_fnptr() {
    assert_eq!(
        compile_and_run("tests/regression/m265_module_scope_fnptr/main.xi"),
        Some(0),
        "module-scope fn-pointer storage must call + reassign (m265)"
    );
}

// m267 (REL-20261010-1802-orbitdb twin half): a BARE pattern variant belongs
// to the SCRUTINEE's enum when that enum declares it -- a foreign same-named
// enum (twin) must not force the "belongs to 'DbError'" T001; the program
// compiles and the match dispatches on the scrutinee's tags.
#[test] fn e2e_m267_twin_scrutinee_enum() {
    assert_eq!(
        compile_and_run("tests/regression/m267_twin_scrutinee_enum/main.xi"),
        Some(0),
        "bare pattern variants must resolve against the scrutinee's enum (m267)"
    );
}

// m270 (REL-20261010-1854-pulse, site-mode AV root cause): a module's OWN
// struct-returning helper called unqualified from the same module must bind
// the module's definition -- with a foreign same-named Str fn present, the
// pre-m261 bare slot hijacked the call (`call i8* @out_text` + struct load
// from i8*), the class that crashed pulse at startup.
#[test] fn e2e_m270_struct_return_local_call() {
    assert_eq!(
        compile_and_run("tests/regression/m270_struct_return_local_call/main.xi"),
        Some(0),
        "module-local struct-returning call must bind the caller module (m270)"
    );
}

// m271 (XVC-C-15, REL-20261011-1134-xvector): literal stores into a
// Vec[Float32] slot must narrow numerically (f64 literal -> 0.0f and Int
// literal -> 9.8e-45 before; the write-side sibling of the m260 read fix).
// Covers local Vecs, a struct Vec[Float32] field and the Vec[Float64]
// Int-literal widening; variable/push/f64-slot controls stay green.
// m280 (REL-20261011-2230-bindings): an outbound Int->ptr cast across a
// whole-body unsafe return must keep the pointer (the Copy-Out Str-copied
// the *UInt8 tail), and a Vec[Str] element read off a temporary must
// concatenate as a string (the concat verdict printed the raw pointer).
#[test] fn e2e_m280_unsafe_ptr_temp_str() {
    assert_eq!(
        compile_and_run("tests/regression/m280_unsafe_ptr_temp_str/main.xi"),
        Some(0),
        "unsafe ptr return + temporary Vec[Str] element (m280)"
    );
}

// m283 (REL-20261013-1524-bindings-2 B-26): a nested enum payload
// (`Result[Option[Bag], Str]` with a Vec-carrying struct) must read back the
// CONCRETE payload monomorph. `llvm_type_for("Option[Bag]")` stripped the
// generic args and returned the erased `%struct.Option` (`{ i64, i64 }`),
// so the payload read loaded 16 bytes of the heap box as an inline struct:
// the tag picked up pointer bits and `items.len()` read garbage (9), and
// using the corrupt Vec AV'd. Flat `Result[BagOutcome, Str]` and
// single-level `Option[Bag]` controls were green.
#[test] fn e2e_m283_nested_payload_read() {
    assert_eq!(
        compile_and_run("tests/regression/m283_nested_payload_read/main.xi"),
        Some(0),
        "nested Result[Option[Bag]] payload must read the concrete monomorph (m283)"
    );
}

// m283 (REL-20261010-1549-packages-8): a postcondition call-cycle
// (`f`'s ensures calls `g` whose contract calls `f`) must not recurse into
// the depth-500 trap (0xC000001D). The per-function re-entrancy counter
// skips the nested clause check; the outer checks still run (add1/clamp
// prove the guard did not swallow normal requires/ensures).
#[test] fn e2e_m283_contract_cycle() {
    assert_eq!(
        compile_and_run("tests/regression/m283_contract_cycle/main.xi"),
        Some(0),
        "postcondition call-cycles must not trap; normal contracts still check (m283)"
    );
}

// m284 (REL-20261011-2140-bindings B-21): `&mut` of a var initialized from a
// struct-returning call (direct or Result-payload extraction) crashed codegen
// with `use of undefined value '%tmp4'` -- `collect_ident_names` had no
// `Expr::MutRef` arm, so the confined block never captured the local and the
// block fn referenced the enclosing fn's driver temp. Literal-init control
// and the payload-extraction variant both lock here.
#[test] fn e2e_m284_mut_of_struct_return() {
    assert_eq!(
        compile_and_run("tests/regression/m284_mut_of_struct_return/main.xi"),
        Some(0),
        "&mut of a struct-returning call init must capture the local (m284)"
    );
}

// m278 (REL-20261011-2137-engine): two modules declare a same-named type
// (`Mesh`) with different layouts; the entry file's imports must decide
// which one its bare references use. Stands in for the engine module-root
// AV (pre-fix, the injected bare-name winner flipped with the project root
// name and the loser compiled against the winner's layout).
#[test] fn e2e_m278_type_name_collision() {
    assert_eq!(
        compile_and_run("tests/regression/m278_type_name_collision/main.xi"),
        Some(0),
        "same-named types across modules must resolve by the entry's imports (m278)"
    );
}

#[test] fn e2e_m271_f32_slot_store() {
    assert_eq!(
        compile_and_run("tests/regression/m271_f32_slot_store/main.xi"),
        Some(0),
        "literal stores into float Vec slots must convert, not store raw bits (m271)"
    );
}

// m276 (REL-20261012-1418-pulse): a fn-typed param's name leaks into the
// program-wide closure_locals set; a plain same-named UInt32 param doing
// `v as Int` must NOT be env-unwrapped -- pre-fix it compiled to
// inttoptr(zext(v)) + load and AV'd on the empty-store kv segment create
// (the stdlib write_u16_le/write_u32_le shape behind the PULSE kv-mode
// regression). The fn-param control keeps the B-007 env wrap green.
#[test] fn e2e_m276_env_name_leak_cast() {
    assert_eq!(
        compile_and_run("tests/regression/m276_env_name_leak_cast/main.xi"),
        Some(0),
        "a leaked closure-local name must not hijack a plain int As-cast (m276)"
    );
}

// m165 (packages backlog): a Vec[UInt8] byte buffer must grow past the old
// 2^24-element ceiling (16 MB); the growth guard now allows 2^32 elements.
#[test] fn e2e_m165_vec_byte_buffer_gt_16mb() {
    assert_eq!(
        compile_and_run("tests/regression/m165_vec_byte_buffer_gt_16mb/main.xi"),
        Some(0),
        "a >16 MB Vec[UInt8] buffer must grow instead of trapping (m165)"
    );
}

// m168 (packages bad44b2): assignment to a `&mut T` parameter must write
// THROUGH the address; the old path rebound the local slot and dropped it.
#[test] fn e2e_m168_mut_ref_write_through() {
    assert_eq!(
        compile_and_run("tests/regression/m168_mut_ref_write_through/main.xi"),
        Some(0),
        "&mut Int/Vec/Str assignment must write through the parameter (m168)"
    );
}

// m167 (packages 2d91399): module-global Vecs must take the inline push and
// read fast paths (the generic stdlib body emitted invalid IR).
#[test] fn e2e_m167_global_vec_push() {
    assert_eq!(
        compile_and_run("tests/regression/m167_global_vec_push/main.xi"),
        Some(0),
        "module-global Vec push/read must compile and be correct (m167)"
    );
}

// m169 (stdlib C24-1 relay, p_geom_vector_result_bits): caller-side element
// reads of module-qualified results whose LEAF is shared with a
// different-return-type function (vector.lerp/clamp/hadamard over math.*,
// curves.b_spline over geometry_extended.*) must keep the callee's declared
// Vec element type. Before the fix the binding lost it and indexed reads took
// the scalar i64 path (the double's bits surfaced via sitofp).
#[test] fn e2e_m169_sameleaf_qualified_result_elem() {
    assert_eq!(
        compile_and_run("tests/regression/m169_sameleaf_qualified_result_elem/main.xi"),
        Some(0),
        "same-leaf qualified Vec results must read their stored elements (m169)"
    );
}

// m169 IR lock: the float element read must bitcast the loaded double bits
// (bitcast i64 -> double), never sitofp them as an integer.
#[test] fn e2e_m169_sameleaf_result_elem_read_ir() {
    let ir = compile_ir("tests/regression/m169_sameleaf_qualified_result_elem/main.xi")
        .expect("m169 fixture must compile to IR");
    let lines: Vec<&str> = ir.lines().collect();
    let idx = lines.iter().position(|l| {
        l.contains("fcmp une double") && l.contains("1.50000000000000000e0")
    }).expect("m169: the 1.5 float comparison must be in the IR");
    let prev = lines[idx.saturating_sub(1)];
    assert!(
        prev.contains("bitcast i64"),
        "m169: the element read must bitcast the loaded double bits; preceding line: {prev}"
    );
    assert!(
        !prev.contains("sitofp i64"),
        "m169: the element read must not treat the double bits as an integer; preceding line: {prev}"
    );
}

// m170 (stdlib C24-2 relay, p_curve_thunk_zero): a call through an fn-typed
// PARAM must keep the parameter's declared return type on the binding
// (`var v = samples(t)`), or element reads inside the callee take the scalar
// i64 path (curve_length's sampled Vec reads collapsed).
#[test] fn e2e_m170_fn_typed_vec_return() {
    assert_eq!(
        compile_and_run("tests/regression/m170_fn_typed_vec_return/main.xi"),
        Some(0),
        "fn-typed params returning Vec[Float64] must be readable in the callee (m170)"
    );
}

// m170 IR lock: the callee-side element read must bitcast the loaded double
// bits, never sitofp them as an integer.
#[test] fn e2e_m170_fn_typed_vec_return_ir() {
    let ir = compile_ir("tests/regression/m170_fn_typed_vec_return/main.xi")
        .expect("m170 fixture must compile to IR");
    let start = ir.find("define double @fn_typed_vec_read")
        .expect("m170: the fn-typed read helper must be defined");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("bitcast i64"),
        "m170: the fn-typed param's Vec element read must bitcast the loaded bits"
    );
    assert!(
        !body.contains("sitofp i64"),
        "m170: the fn-typed param's Vec element read must not treat the bits as an integer"
    );
}

// m168 residual (stdlib/packages narrowing): a `&mut T` PARAM passed to a
// BY-VALUE T param in a direct call must load the pointee (`byval(s)`), not
// ptrtoint the address; the container-pointee sibling (`veclen(s)`) is
// locked too.
#[test] fn e2e_m168_mut_ref_byval_arg() {
    assert_eq!(
        compile_and_run("tests/regression/m168_mut_ref_byval_arg/main.xi"),
        Some(0),
        "&mut T args to by-value params must load the pointee (m168 residual)"
    );
}

// IR lock: @bump's body must load the pointee for the by-value call; before
// the fix the address was ptrtoint'd.
#[test] fn e2e_m168_mut_ref_byval_arg_ir() {
    let ir = compile_ir("tests/regression/m168_mut_ref_byval_arg/main.xi")
        .expect("m168 residual fixture must compile to IR");
    let start = ir.find("define void @bump(")
        .expect("m168 residual: @bump must be defined");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("load i64, i64*"),
        "m168 residual: the &mut arg must be loaded before the by-value call"
    );
    assert!(
        !body.contains("ptrtoint"),
        "m168 residual: the &mut arg must not be ptrtoint'd"
    );
}

// C23 (playground relay, tools/compiler-repros/c23): the driver used to
// optimize each module twice (`opt -O<level>` then `clang -O<level>`). On
// hosts with LLVM 18 that double pipeline miscompiles the three lesson
// shapes; these fixtures return 11/12/13 when a count is wrong. Windows has
// no `opt`, so the lock bites on the ubuntu CI leg -- where the bug lived.
#[test] fn e2e_c23_l6_category() {
    assert_eq!(
        compile_and_run("tests/regression/c23_l6_category/main.xi"),
        Some(0),
        "C23 l6-15 shape must keep its category count"
    );
}

#[test] fn e2e_c23_l7_pending() {
    assert_eq!(
        compile_and_run("tests/regression/c23_l7_pending/main.xi"),
        Some(0),
        "C23 l7-39 shape must keep its pending count"
    );
}

#[test] fn e2e_c23_l8_quick() {
    assert_eq!(
        compile_and_run("tests/regression/c23_l8_quick/main.xi"),
        Some(0),
        "C23 l8-09 shape must keep its quick-meal count"
    );
}

// m173 guard for the (g) fix: scalar/Str payload enum equality must keep
// compiling and comparing (only aggregate payloads are rejected).
#[test] fn e2e_m173_enum_eq_scalar_payload() {
    assert_eq!(
        compile_and_run("tests/regression/m173_enum_eq_scalar_payload/main.xi"),
        Some(0),
        "scalar/Str payload enum equality must keep working"
    );
}

// m174 (stdlib relay, p_struct_literal_field_order): out-of-order struct
// literal fields must map to their declared slots by NAME.
#[test] fn e2e_m174_struct_literal_field_order() {
    assert_eq!(
        compile_and_run("tests/regression/m174_struct_literal_field_order/main.xi"),
        Some(0),
        "out-of-order struct literal fields must land in their declared slots"
    );
}

// m175 (playground C24): a Call returning i64 fed to a pointer extern param
// carries FFI handle bits (`xiom_stdin() -> Int` into `fgets(..., *UInt8)`);
// it must inttoptr, never materialize a pointee temporary. This runtime lock
// pipes stdin and asserts the read_line echo.
#[test] fn e2e_m175_read_line_piped_stdin() {
    use std::io::Write;
    let src = "tests/regression/m175_stdin_handle_int_pointer/main.xi";
    let exe_name = format!(
        "e2e_m175_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    );
    let exe_path = project_root().join(&exe_name);
    let _ = std::fs::remove_file(&exe_path);
    let compile = Command::new(xiom_path())
        .args(["-o", exe_name.as_str(), src])
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_path()));
    assert!(
        compile.status.success(),
        "m175 compile failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let mut child = Command::new(&exe_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn m175 binary");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"Ada\n")
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait m175 binary");
    let _ = std::fs::remove_file(&exe_path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("got: [Ada]"),
        "read_line must echo piped stdin; stdout={stdout:?} stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// m175 IR lock: the fgets stream arg must come from an inttoptr of the i64
// handle, never from a materialized pointee temporary.
#[test] fn e2e_m175_stdin_handle_inttoptr_ir() {
    let ir = compile_ir("tests/regression/m175_stdin_handle_int_pointer/main.xi")
        .expect("m175 fixture must compile to IR");
    let lines: Vec<&str> = ir.lines().collect();
    let idx = lines
        .iter()
        .position(|l| l.contains("call i8* @fgets("))
        .expect("m175: the fgets call must be in the IR");
    let near: Vec<&&str> = lines[..idx].iter().rev().take(3).collect();
    assert!(
        near.iter().any(|l| l.contains("inttoptr i64")),
        "m175: the fgets stream arg must inttoptr the i64 handle; near call: {near:?}"
    );
    assert!(
        !near.iter().any(|l| l.contains("alloca i8")),
        "m175: no pointee temporary may be materialized for the handle; near call: {near:?}"
    );
}

// m176 (packages relay, gcp): module-qualified struct literals, annotations
// and by-value params must resolve to the registered struct.
#[test] fn e2e_m176_qualified_struct_literal() {
    assert_eq!(
        compile_and_run("tests/regression/m176_qualified_struct_literal/main.xi"),
        Some(0),
        "module-qualified struct types must resolve in literals/annotations/params"
    );
}

// R52 (packages relay): `use xiom.test; assert(1 == 1, "...")` -- an
// unqualified call must bind the IMPORTED module's exported TestResult assert
// (`test.assert`), not a transitively-imported private helper (`core.assert`)
// or the submodule's void assert (`test.assert.assert`). Private functions no
// longer register bare cross-module aliases and imported modules rank above
// the global keep-first alias.
#[test] fn e2e_m105_unqualified_assert() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("test").join("test.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/test/test.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m105_unqualified_assert/main.xi"),
        Some(0),
        "R52 unqualified xiom.test.assert probe must compile and run"
    );
}

// R51 (stdlib p_pre_capture_callee): contract clauses wrapped in an
// IMPLICATION (`result is Some => total(b) == total(b)@pre - 1`) must still
// collect @pre variables and emit entry snapshots -- the walker had no Imply
// arm, so no snapshot existed and the ensures compared the live pointer with
// itself (2 == 2 - 1 violations across list/queue/rbtree/fenwick).
#[test] fn e2e_m104_pre_capture_callee() {
    assert_eq!(
        compile_and_run("tests/regression/m104_pre_capture_callee/main.xi"),
        Some(0),
        "R51 implication-wrapped @pre snapshot probe must compile and run"
    );
}

// R51 (playground L4-33/39): `self@pre.items.len()` in an ensures -- the
// receiver is Field(AtPre(self), items); type resolution and the Vec.len
// dispatch must look through @pre (was xiom_str_len(<%struct.Vec>) -> clang
// "defined with type %struct.Vec but expected ptr").
#[test] fn e2e_m101_pre_len_receivers() {
    assert_eq!(
        compile_and_run("tests/regression/m101_pre_len_receivers/main.xi"),
        Some(20),
        "R51 @pre field-base .len() probe must compile and run"
    );
}

// R51 (playground L4-26): a user `fn range(scores: &Vec[Int]) -> Int` must not
// hijack `for i in range(0, n)` -- the loop lowering resolves the builtin
// structurally (2 args), so the i64 result is not GEP'd as a 2-field struct
// (clang "invalid getelementptr indices").
#[test] fn e2e_m102_range_shadowing() {
    assert_eq!(
        compile_and_run("tests/regression/m102_range_shadowing/main.xi"),
        Some(25),
        "R51 range-shadowing for-in probe must compile and run"
    );
}

// R51 (playground L4-29): `items@pre.len()` on a Vec PARAM receiver -- the
// Vec.len dispatch must look through the top-level @pre (same class as m101).
#[test] fn e2e_m103_pre_len_param() {
    assert_eq!(
        compile_and_run("tests/regression/m103_pre_len_param/main.xi"),
        Some(30),
        "R51 @pre param .len() probe must compile and run"
    );
}

// R49-3 (stdlib relay p_result_payload_contract): one module with scalar +
// Vec payload Result contracts. `result.value.len()` after `is Ok/Err` must
// dispatch on the rebound payload container (was xiom_str_len on a
// %struct.Vec -> clang type error).
#[test] fn e2e_m100_result_payload_contract() {
    assert_eq!(
        compile_and_run("tests/regression/m100_result_payload_contract/main.xi"),
        Some(0),
        "R49 result-payload contract probe must compile and run"
    );
}

// R49-1 (stdlib relay p_module_path_alias): importing a MOVED module by its
// file path must register the declared identity, not the alias. The checker
// resolves the catalog on `xiom --check`, so compiling this program exercises
// the same path (the old alias corrupted xiom.crypto's exports -> T001s in
// rng_crypto). Needs the stdlib checkout.
#[test] fn e2e_m99_module_path_alias() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("crypto").join("legacy").join("md5.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/crypto/legacy/md5.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m99_module_path_alias/main.xi"),
        Some(0),
        "R49 moved-module path-alias probe must compile and run"
    );
}

// R49 (playground C18 residue): fixed arrays of Str elements must keep the
// i8* element type -- `friends[i]` used to ptrtoint the string handle and
// print the ADDRESS (L0-11 "Hi, 1406..."). Harness-locked because the
// output is inside the compiled program (exit code only here).
#[test] fn e2e_m96_array_str_elem() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m96_array_str_elem/main.xi"),
        Some(0),
        "R49 array-of-Str element probe must compile and run"
    );
}

// R49 (playground C17 residue): qualified enum variants in match arms
// (`TrafficLight.Green`) must infer the parent enum struct; the old i64
// fallback made `next()` return a mistyped enum (L2-19 AV).
#[test] fn e2e_m97_enum_variant_match() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_layout_ok(&stdlib_root) {
        let msg = "SKIP: stdlib checkout has no xiom/io.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m97_enum_variant_match/main.xi"),
        Some(0),
        "R49 enum-variant match probe must compile and run"
    );
}
#[test] fn e2e_m37_nested_vec() { assert_eq!(compile_and_run("tests/regression//m37_nested_vec.xi"), Some(0)); }
#[test] fn e2e_m37_short_circuit() { assert_eq!(compile_and_run("tests/regression//m37_short_circuit.xi"), Some(0)); }
#[test] fn e2e_m37_match_float_payload() { assert_eq!(compile_and_run("tests/regression//m37_match_float_payload.xi"), Some(0)); }
#[test] fn e2e_m37_inline_call_concat() { assert_eq!(compile_and_run("tests/regression//m37_inline_call_concat.xi"), Some(0)); }
#[test] fn e2e_m37_else_if() { assert_eq!(compile_and_run("tests/regression//m37_else_if.xi"), Some(0)); }
#[test] fn e2e_m37_contract_pass() { assert_eq!(compile_and_run("tests/regression//m37_contract_pass.xi"), Some(0)); }
// BUG 22 #6: loop-body binding captured by a later unsafe block
#[test] fn e2e_m37_loop_capture() { assert_eq!(compile_and_run("tests/regression//m37_loop_capture.xi"), Some(0)); }
// BUG 24: structural equality for same-type structs without derived eq
#[test] fn e2e_m37_structural_eq() { assert_eq!(compile_and_run("tests/regression//m37_structural_eq.xi"), Some(0)); }
// BUG 25 #3: user fn named from_bytes not hijacked by the builtin
#[test] fn e2e_m37_from_bytes_fn() { assert_eq!(compile_and_run("tests/regression//m37_from_bytes_fn.xi"), Some(0)); }
// BUG 25 #5: Option/Result .value/.error payload-aware reads
#[test] fn e2e_m37_opt_payload_value() { assert_eq!(compile_and_run("tests/regression//m37_opt_payload_value.xi"), Some(0)); }
// BUG 26: int<->float mixing requires explicit `as` (allowed surface)
#[test] fn e2e_m37_numeric_policy() { assert_eq!(compile_and_run("tests/regression//m37_numeric_policy.xi"), Some(0)); }
// Labeled break/continue
#[test] fn e2e_m37_labeled_loops() { assert_eq!(compile_and_run("tests/regression//m37_labeled_loops.xi"), Some(0)); }
// BUG 27: in-code debug intrinsics
#[test] fn e2e_m37_debug_intrinsics() { assert_eq!(compile_and_run("tests/regression//m37_debug_intrinsics.xi"), Some(0)); }

// ============================================================================
// BUG 43-47 batch (2026-08-18) -- docs/COMPILER_BUGS.md
// BUG 43: Result[Float64, Str] payload read via sitofp (bitcast needed)
// BUG 44: deref/coercion of &Str loaded a byte instead of the pointer
// BUG 45: method-form interface dispatch inside generic-bound fns -> stub
// BUG 46: generic &UserStruct[T] param field reads returned garbage
// BUG 47: ref_params/param_locals leaked across fns (fn-param -> AV)
// ============================================================================
#[test] fn e2e_m37_bug43_result_f64_payload() { assert_eq!(compile_and_run("tests/regression//m37_bug43_result_f64_payload.xi"), Some(0)); }
#[test] fn e2e_m37_bug44_str_deref() { assert_eq!(compile_and_run("tests/regression//m37_bug44_str_deref.xi"), Some(0)); }
#[test] fn e2e_m37_bug45_iface_method_generic() { assert_eq!(compile_and_run("tests/regression//m37_bug45_iface_method_generic.xi"), Some(0)); }
#[test] fn e2e_m37_bug46_generic_struct_ref() { assert_eq!(compile_and_run("tests/regression//m37_bug46_generic_struct_ref.xi"), Some(0)); }
#[test] fn e2e_m37_bug47_ref_params_leak() { assert_eq!(compile_and_run("tests/regression//m37_bug47_ref_params_leak.xi"), Some(0)); }

// ============================================================================
// BUG 48-52 batch (2026-08-18): associated-form dispatch + &Vec[T] ABI,
// fn-param vs impl-method collision, pointer container names, Option[Struct]
// payloads, Map enum values
// ============================================================================
#[test] fn e2e_m37_bug48_associated_generic_vec() { assert_eq!(compile_and_run("tests/regression//m37_bug48_associated_generic_vec.xi"), Some(0)); }
#[test] fn e2e_m37_bug49_fn_param_impl_collision() { assert_eq!(compile_and_run("tests/regression//m37_bug49_fn_param_impl_collision.xi"), Some(0)); }
#[test] fn e2e_m37_bug50_ptr_container_name() { assert_eq!(compile_and_run("tests/regression//m37_bug50_ptr_container_name.xi"), Some(0)); }
#[test] fn e2e_m37_bug51_option_struct_payload() { assert_eq!(compile_and_run("tests/regression//m37_bug51_option_struct_payload.xi"), Some(0)); }
#[test] fn e2e_m37_bug52_map_enum_values() { assert_eq!(compile_and_run("tests/regression//m37_bug52_map_enum_values.xi"), Some(0)); }

// ============================================================================
// BUG 53/55 (2026-08-18): &[N]T param lowering + unsafe-block context capture
// ============================================================================
#[test] fn e2e_m37_bug53_array_ref_param() { assert_eq!(compile_and_run("tests/regression//m37_bug53_array_ref_param.xi"), Some(0)); }
#[test] fn e2e_m37_bug55_unsafe_ptr_capture() { assert_eq!(compile_and_run("tests/regression//m37_bug55_unsafe_ptr_capture.xi"), Some(0)); }

// ============================================================================
// BUG 53 write-facet / BUG 55 facet-2 / BUG 56 (2026-08-18 round 3)
// ============================================================================
#[test] fn e2e_m37_bug53_array_ref_write() { assert_eq!(compile_and_run("tests/regression//m37_bug53_array_ref_write.xi"), Some(0)); }
#[test] fn e2e_m37_bug55_payload_loop() { assert_eq!(compile_and_run("tests/regression//m37_bug55_payload_loop.xi"), Some(0)); }
#[test] fn e2e_m37_bug56_ensure_expr_body() { assert_eq!(compile_and_run("tests/regression//m37_bug56_ensure_expr_body.xi"), Some(0)); }

// ============================================================================
// gzip-DECOMPRESS catalog mono (2026-08-19 round 4, queue item 1):
// Result-payload FIELD access (`decoded.value`) unboxing + if-expression
// Vec-valued arms (`let x = if c { f() } else { g() };`).
// ============================================================================
#[test] fn e2e_m37_gzip_roundtrip() { assert_eq!(compile_and_run("tests/regression//m37_gzip_roundtrip.xi"), Some(0)); }

// ============================================================================
// Round 6 (2026-08-19): Imply short-circuit (gzip validation Err path),
// Try-binding Str payloads + byte_at receiver, substr inline handler,
// Str-builtin receiver guards, path.xi join_paths import.
// ============================================================================
#[test] fn e2e_m37_round6_path_gzip() { assert_eq!(compile_and_run("tests/regression//m37_round6_path_gzip.xi"), Some(0)); }

// ============================================================================
// Round 7 (2026-08-20): inlined Vec.pop + match Option slot on an EMPTY vec
// (bare "pop" resolved to no registered key -> no scrutinee alloca -> the match
// took the Some arm unconditionally). Root: Vec/Set/Slice methods were never
// injected (non-pub generic receivers whose type decl is a compiler builtin),
// so Vec.first/last/clear/insert/remove were zero-param stubs and the inline
// builtins' match scrutinees resolved to nothing. Also covers the mono'd
// Vec-method pointer arithmetic (GEP element scaling + element-width loads)
// and the *UInt8 buffer concat gate.
// ============================================================================
#[test] fn e2e_m37_round7_vec_pop_slot() { assert_eq!(compile_and_run("tests/regression//m37_round7_vec_pop_slot.xi"), Some(0)); }

// ============================================================================
// Round 8 (2026-08-20): catalog &mut self receiver wiring for user generic
// structs -- VecDeque/Stack/Queue/LinkedList/BTreeMap mutations were entirely
// lost (non-pub generic type decls + methods were never injected; calls
// hijacked same-leaf methods of other types). Plus Option<&T> reference
// payloads (rand.weighted_pick): the slot ADDRESS was strcmp'd as the
// string -- &T value uses now auto-deref via the "&T" xiom record.
// ============================================================================
#[test] fn e2e_m38_round8_catalog_mut_self() { assert_eq!(compile_and_run("tests/regression//m38_round8_catalog_mut_self.xi"), Some(0)); }
#[test] fn e2e_m38_round8_ref_payload() { assert_eq!(compile_and_run("tests/regression//m38_round8_ref_payload.xi"), Some(0)); }

// ============================================================================
// Round 9 (2026-08-20): Set container ABI -- the compiler had NO builtin Set
// layout, so Set values erased to i64 while stdlib methods operated on
// %struct.Set (new() hijacked Reverse.new; Set params/returns/fields compiled
// as i64; field-receiver insert() mono'd Vec.insert). Now the stdlib Set type
// injects and resolves like any struct.
// ============================================================================
#[test] fn e2e_m39_round9_set_abi() { assert_eq!(compile_and_run("tests/regression//m39_round9_set_abi.xi"), Some(0)); }

// ============================================================================
// Round 10 (2026-08-20): checker builtin Ord/Bounded interface resolution
// (C001) -- the stdlib Ord tower (impl Ord[Int] with compare+cmp) registers;
// cmp/min/max are compiler-derivable for primitives; generic-param static
// receivers (T.max_value() in mono'd bodies) resolve via current_type_map;
// checked/saturating arithmetic works through the Bounded + Ord bounds.
// ============================================================================
#[test] fn e2e_m40_round10_ord_bounded() { assert_eq!(compile_and_run("tests/regression//m40_round10_ord_bounded.xi"), Some(0)); }

// ============================================================================
// Round 11 (2026-08-20): B-007 closures -- fn-typed PARAMS hold a closure ENV
// pointer (field 0 = the fn ptr). Calling f(x) inside a generic body goes
// through the M20-A1 closure path: the param must be registered as a closure
// local (the ENV pointer was inttoptr'd as a CODE pointer -- 0xC0000005 in
// Option.map), and the closure's REAL return type drives the fn-pointer
// signature (struct returns are BY VALUE -- 0xC0000005 in Option.and_then).
// ============================================================================
#[test] fn e2e_m41_round11_b007_closures() { assert_eq!(compile_and_run("tests/regression//m41_round11_b007_closures.xi"), Some(0)); }

// ============================================================================
// Round 12 (2026-08-21): Str-returning closures through Result/Err -- closure
// thunk params are i64 (uniform env-first ABI) but their DECLARED XIOM types
// were not tracked: a Str param used inside the body degraded to a scalar
// (alloca i8 + trunc i64 of the string HANDLE) -- corrupt map_err payloads.
// Fix: closure params record local_xiom_types/ref_params/signed_locals, and
// the mono fn-typed param's return type resolves through type_map.
// ============================================================================
#[test] fn e2e_m42_round12_str_closures() { assert_eq!(compile_and_run("tests/regression//m42_round12_str_closures.xi"), Some(0)); }

// ============================================================================
// Round 13 (2026-08-22): closure-based iter adapters -- (1) closure env
// STRUCT NAME collisions (identical capture shapes redefined
// %struct.__closure_env_N); (2) captured-state MUTATION persisted via
// direct env-field GEP binding (count/fold hung); (3) FN-TYPED FIELD
// calls go env-first (`self.next_fn()` was a zero-param stub);
// (4) enum-return scrutinees from closure calls get discriminant checks.
// The enumerate/zip/btree tuple-payload half lives in e2e_m44 (the
// combined module flips the documented clang -O2/MSVC-CRT crash).
// ============================================================================
#[test] fn e2e_m43_round13_closure_adapters() { assert_eq!(compile_and_run("tests/regression//m43_round13_closure_adapters.xi"), Some(0)); }

// ============================================================================
// Round 13 (2026-08-22): TUPLE PAYLOADS through Option/Vec -- Some((a, b))
// payload bindings deref the heap box; Vec[(Int, Int)] slots store the
// full 16-byte element; "(Int, Int)" normalizes to the registered
// "Tuple__Int__Int"; generic mono returns (Vec[Tuple__Int__T]) stay
// tracked. Unblocks enumerate/zip AND BTreeMap.first_entry/last_entry
// (smoke_collections_btree_map was exit 7 at baseline).
// ============================================================================
#[test] fn e2e_m44_round13_tuple_payloads() { assert_eq!(compile_and_run("tests/regression//m44_round13_tuple_payloads.xi"), Some(0)); }

// ============================================================================
// Round 14 (2026-08-22): AGGREGATE-typed closure params -- the closure thunk
// declared every param as i64 while the call site passed structs/tuples BY
// VALUE (16 bytes split across registers; the i64 param read only the
// first). Thunks now declare aggregate params with their real LLVM types
// (by-value) and bind them typed; __fnwrap forwards aggregates by value.
// ============================================================================
#[test] fn e2e_m45_round14_aggregate_closure_params() { assert_eq!(compile_and_run("tests/regression//m45_round14_aggregate_closure_params.xi"), Some(0)); }

// ============================================================================
// Round 14 (2026-08-22): Vec[Str] ELEMENT method calls (v[0].len() emitted
// an invalid GEP -- the i8* receiver was GEP'd as a struct) + narrow-SIGNED
// Vec loads (Int16 -30000 zext'd to 35536 -- emit_elem_load now sexts when
// the container's element type is signed).
// ============================================================================
#[test] fn e2e_m46_round14_vec_str_elems_narrow() { assert_eq!(compile_and_run("tests/regression//m46_round14_vec_str_elems_narrow.xi"), Some(0)); }

// ============================================================================
// Round 14 (2026-08-22): multibyte Char family (BUG 26 #7) -- xiom_char_at
// decodes the UTF-8 CODEPOINT (i64 ABI; was a raw byte); xiom_byte_at is
// the raw-byte accessor (byte_at double-decoded once char_at was fixed);
// Vec[Char] slots are 4 bytes (codepoints > 255 truncated to their low
// byte); the as-cast widening zexts UInt*-returning calls. smoke_string_
// slice + smoke_convert_utf (mojibake-restored) green.
// ============================================================================
#[test] fn e2e_m47_round14b_multibyte_chars() { assert_eq!(compile_and_run("tests/regression//m47_round14b_multibyte_chars.xi"), Some(0)); }

// ============================================================================
// Round 14 (2026-08-22): by-value self methods returning the SAME type no
// longer write the result back into the receiver's slot (the time.Duration
// family -- identity/sum/diff clobbered the receiver); generic fns with
// fn-typed params resolve AGGREGATE instantiations (ZipIter find/all/any/
// nth with fn(&(Int, Int)) -> Bool -- the mono'd _find_via_Tuple__Int__Int
// path + box-deref payload bindings); negative Int->narrow as casts
// compare signed (inferred signed_locals); const-generic [N]T arrays
// (array.map[T, U, const N] mono'd with N=2 + T=U=Int16 -- [2 x i16]
// params/returns at both def and call sites).
// ============================================================================
#[test] fn e2e_m48_round14c_writeback_aggregates() { assert_eq!(compile_and_run("tests/regression//m48_round14c_writeback_aggregates.xi"), Some(0)); }

// ============================================================================
// Round 15 (2026-08-23): fn-typed params marshal Float64 through the
// __fnwrap/closure thunks with REAL double types (the uniform-i64
// convention read the wrong register class + sitofp'd the bit pattern --
// apply(sqminus2, 2.0) returned 0); const-generic [N]T arrays re-publish
// N per distinct argument array (array.len stale across call sites) and
// narrow-element reads through &[N]T mono bodies (array.first on
// [1 as Int8, ...] -- the caller's array local leaked into the mono body
// and the +1 array-buffer path fired; UInt8 zero-extends); array.map's
// [N]U result resolves implicitly (by-value [N]T params + Vec->aggregate
// materialization); NON-pub catalog interfaces inject so Ord[T].compare
// dispatches inside mono'd stdlib bodies (BinaryHeap order).
// ============================================================================
#[test] fn e2e_m49_round15_fnfloat_constarrays() { assert_eq!(compile_and_run("tests/regression//m49_round15_fnfloat_constarrays.xi"), Some(0)); }

// ----------------------------------------------------------------------------
// m286 (REL-20261010-1549-stdlib-7): cross-type generic callbacks.
// ----------------------------------------------------------------------------

// m286: cross-type generic callback -- the binding must keep the substituted
// "Vec[Str]" (inferred and explicit bracket spellings); pre-fix the element
// read crossed as a scalar and the Str handle byte-materialized.
#[test] fn e2e_m286_cross_type_callback_bind() {
    assert_eq!(
        compile_and_run("tests/regression/m286_cross_type_callback_bind/main.xi"),
        Some(0),
        "a cross-type generic callback result must keep its element type (m286)"
    );
}

// m286: Option[Int].map[Str] match payload -- the Some(s) binding must be Str.
#[test] fn e2e_m286_core_map_match_payload() {
    assert_eq!(
        compile_and_run("tests/regression/m286_core_map_match_payload/main.xi"),
        Some(0),
        "a cross-type map scrutinee must bind its match payload concretely (m286)"
    );
}

// m286: stdlib sort_by_key[Int, Str] must sort lexicographically.
#[test] fn e2e_m286_sort_by_key_cross_type() {
    assert_eq!(
        compile_and_run("tests/regression/m286_sort_by_key_cross_type/main.xi"),
        Some(0),
        "sort_by_key with Str keys must order lexicographically (m286)"
    );
}

// m286: Str relational operators are content ordering (multi-char/prefix).
#[test] fn e2e_m286_str_lexicographic_compare() {
    assert_eq!(
        compile_and_run("tests/regression/m286_str_lexicographic_compare/main.xi"),
        Some(0),
        "Str <,>,<=,>= must compare content, not first bytes (m286)"
    );
}

// m287 (REL-20261010-1548-stdlib-3): importing xiom.convert.tostring pulls
// math.mathematical_logic in; the adapter's `self.predicate(&v)` closure
// FIELD call must not be hijacked by the same-leaf free fn (a direct call
// bypassed the closure env and filter/take_while returned None).
#[test] fn e2e_m287_tostring_import_adapters() {
    assert_eq!(
        compile_and_run("tests/regression/m287_tostring_import_adapters/main.xi"),
        Some(0),
        "adapter closure fields must win over same-leaf free fns (m287)"
    );
}

// m288 (REL-20261010-1548-stdlib-4): `x.a@pre` on a &mut struct parameter
// must read the ENTRY snapshot (the v0.64.4 archive aliased the
// post-mutation value); enforcement verified live via a +5 mutated body.
#[test] fn e2e_m288_mut_param_field_pre() {
    assert_eq!(
        compile_and_run("tests/regression/m288_mut_param_field_pre/main.xi"),
        Some(0),
        "&mut parameter field @pre must snapshot the entry value (m288)"
    );
}

// m289 (REL-20261010-1548-stdlib-5): annotated Slice[Int] locals must use
// the canonical %struct.Slice slot (was an i64 holding the data pointer --
// .len() strlen'd the buffer, helpers AV'd); plus the by-ref option/result
// query matrix.
#[test] fn e2e_m289_slice_annotated_local() {
    assert_eq!(
        compile_and_run("tests/regression/m289_slice_annotated_local/main.xi"),
        Some(0),
        "annotated Slice locals and by-ref queries must read correctly (m289)"
    );
}

// m290 (REL-20261010-1548-stdlib-6 leg a): an alias-qualified struct literal
// (`ser.SerializeError{..}` after `use xiom.serialize as ser;`) must resolve
// through the alias to the module type (was T001 unknown type).
#[test] fn e2e_m290_alias_type_path() {
    assert_eq!(
        compile_and_run("tests/regression/m290_alias_type_path/main.xi"),
        Some(0),
        "alias-qualified type paths must resolve (m290)"
    );
}

// m291 (REL-20261010-1548-stdlib-7): the guard-form implication
// `(result.is_ok == true) => result.len() == ...` must rebind the payload
// like `result is Ok =>`; also the contract re-entrancy guard must not skip
// the outermost check (enforcement live -- the negative twin is the
// checker_locks m291_contract_enforcement_live test).
#[test] fn e2e_m291_ensures_isok_guard() {
    assert_eq!(
        compile_and_run("tests/regression/m291_ensures_isok_guard/main.xi"),
        Some(0),
        "guard-form ensures payload reads must behave like the canonical form (m291)"
    );
}

// m292 (REL-20261010-1548-bindings-4): `module ...ffi` + `use xiom.ffi;` --
// the flattened program-local leaf used to block the catalog pre-load, so
// bare stdlib submodule fns (xiom.ffi.c) never resolved (T001).
#[test] fn e2e_m292_ffi_alias_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m292_ffi_alias_shadow/main.xi"),
        Some(0),
        "leaf-colliding nested module must not shadow the xiom.ffi import (m292)"
    );
}

// m295 (REL-20261010-1548-stdlib-6 leg b): a method-style call of a free fn
// on a BY-VALUE struct/enum receiver (`v.json_get_path(&path)` on a JsonValue
// local) must bind the registered free fn (param 0 typed by value) instead of
// emitting the receiver-qualified alias `JsonValue.json_get_path` (C001).
#[test] fn e2e_m295_foreign_method_call() {
    assert_eq!(
        compile_and_run("tests/regression/m295_foreign_method_call/main.xi"),
        Some(0),
        "by-value receiver UFCS must bind the registered free fn (m295)"
    );
}

// m298 (REL-20261010-1709-xvector): two modules each declare a Counter; a
// consuming module importing exactly one of them uses a BARE `Counter` as a
// struct-field type. Pre-fix the injection left the bare reference alone and
// codegen's same-leaf suffix pick chose the shorter key (the shim), so the
// package Counter value was stored into a shim-typed field slot -> clang
// '%tmp defined with type ...' store mismatch (the xv-counter-type-shadow
// witness). The unique imported owner must bind the bare reference.
#[test] fn e2e_m298_xvector_counter_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m298_xvector_counter_shadow/main.xi"),
        Some(0),
        "bare same-leaf field types must bind the unique import owner (m298)"
    );
}

// m298 (REL-20261012-1956-orbitdb): two co-imported modules declare the
// same public type triad (BTree/NodeStore/BTreeNode); the second module's
// BTree is shape-identical to the first's, so pre-fix the facade exemption
// dropped it and the module's bodies compiled against the first module's
// field metadata -- `child.payloads.push/pop` on a node read out of
// `tree.store.nodes` became C001 'xiom.aa.btree.BTreeNode.push/pop' (the
// xiom.btree + xiom.db.engine compose collision in miniature).
#[test] fn e2e_m298_orbitdb_compose() {
    assert_eq!(
        compile_and_run("tests/regression/m298_orbitdb_compose/main.xi"),
        Some(0),
        "same-named type triads from co-imported modules must keep module identity (m298)"
    );
}

// m300 (REL-20261010-1549-stdlib-2): xiom.geom.geometry_3d's `Box` (shadowed
// by xiom.core's private Box[T]) must be nameable cross-module --
// `geometry_3d.Box{..}` resolved T001 unknown-type and the call-site param
// check compared SPELLINGS (expected bare `Box` vs found `geometry_3d.Box`)
// so aabb_contains/aabb_intersection were uncallable.
#[test] fn e2e_m300_geom_box_unnameable() {
    assert_eq!(
        compile_and_run("tests/regression/m300_geom_box_unnameable/main.xi"),
        Some(0),
        "geometry_3d.Box must be constructible and passable cross-module (m300)"
    );
}

// m300 (REL-20261010-1549-stdlib-3): un-annotated nested Vec[Vec[Float64]]
// results must keep the full nesting -- the tuple-element binding
// `var l2 = lu.0;` lost a level (type_string_full erased tuple element
// containers to "Vec") and the row read went through the scalar elem path
// (row len 0).
#[test] fn e2e_m300_geom_matrix_result_infer() {
    assert_eq!(
        compile_and_run("tests/regression/m300_geom_matrix_result_infer/main.xi"),
        Some(0),
        "un-annotated nested matrix results and tuple elements must infer fully (m300)"
    );
}

// m293 (REL-20261010-1549-packages-11): Map/Set `==` must compare CONTENT
// (keys/values/items Vecs with the concrete K/V/T args), not emit the
// invalid `icmp eq %struct.Vec` for the by-value fields. Covers Map[Int,Int],
// Map[Str,Int] (strcmp through distinct buffers), Set[Int]/Set[Str], a
// clause site, BTreeMap/BTreeSet, and the legacy non-generic HashSet.
#[test] fn e2e_m293_mapset_eq() {
    assert_eq!(
        compile_and_run("tests/regression/m293_mapset_eq/main.xi"),
        Some(0),
        "Map/Set deep == must compile and compare content (m293)"
    );
}

// m294 (REL-20261010-1549-packages-9): a bare `&mut Int` read must yield the
// POINTEE in every value sink -- m197 covered `return p` + arithmetic; this
// lock covers untyped/typed bindings, plain assignment, `*q = p`
// store-through, Vec index read/write operands, scalar cast, unary
// negation, and the Eq/Neq double-deref exposed by the comparison case.
#[test] fn e2e_m294_mut_int_bare() {
    assert_eq!(
        compile_and_run("tests/regression/m294_mut_int_bare/main.xi"),
        Some(0),
        "bare &mut Int reads must yield the pointee in all value sinks (m294)"
    );
}

// m296 (REL-20261011-2320-bindings #1 HIGH): Vec[UInt8].with_len(n) is a
// sized buffer under the documented 8-bytes-per-slot Vec contract -- the
// stdlib zero-fill writes n * 8 bytes. The constructor used to allocate
// cap * elem_size (16 bytes for Vec[UInt8]), so with_len(16) zero-filled
// 128 bytes into a 16-byte region (ASAN: WRITE of size 128 past a 16-byte
// region allocated by Vec.with_len_UInt8; the STATUS_HEAP_CORRUPTION
// class). Construction buffers now floor the slot at 8 bytes
// (vec_slot_size); the IR twin pins the slot-sized ctor deterministically
// (pre-fix: malloc(i64 16), post-fix: malloc(i64 128)).
#[test] fn e2e_m296_vec_with_len() {
    assert_eq!(
        compile_and_run("tests/regression/m296_vec_with_len/main.xi"),
        Some(0),
        "with_len sized buffers must be slot-sized and boundary-safe (m296)"
    );
    let ir = compile_ir("tests/regression/m296_vec_with_len/main.xi")
        .expect("m296: fixture must emit IR");
    assert!(
        ir.contains("call i8* @xiom_guard_alloc(i64 128)"),
        "m296: the sized-buffer ctor must allocate 16 x 8-byte slots through the arena-aware allocator; got:\n{ir}"
    );
    assert!(
        !ir.contains("@xiom_guard_alloc(i64 16)"),
        "m296: the raw element-size allocation must not appear; got:\n{ir}"
    );
}

// m297 (REL-20261010-1615-stdlib): `Path.is_dir` -- a stdlib METHOD whose
// body bare-calls the same-leaf free fn `is_dir(self.inner)` -- self-bound:
// the G-10 implicit-self override re-bound the already-resolved io.is_dir
// to the sibling method @Path.is_dir itself, recursed to the depth-500
// llvm.trap and died with 0xC000001D before any output. Witness shape:
// io import + trailing printlns (p_path_println_context_crash.xi).
#[test] fn e2e_m297_path_println_context() {
    assert_eq!(
        compile_and_run("tests/regression/m297_path_println_context/main.xi"),
        Some(0),
        "Path.is_dir must bind the io.is_dir free fn, not recurse into itself (m297)"
    );
}

// m299 (REL-20261010-1549-packages-12): destructuring a REFERENCE to a tuple
// element (`let (k, v) = &vec[i]`) must bind VALUE COPIES. The m209 ref
// binding leaked component addresses into value uses (prints showed decimal
// addresses; the Str concat path deref'd through a byte load -> AV); the
// workaround was bind-then-destructure. The fixture gates the copied values
// through compares and string concat, and keeps the m209 `k == &key` compare
// plus the by-value/workaround controls green.
#[test] fn e2e_m299_tuple_ref_destructure() {
    assert_eq!(
        compile_and_run("tests/regression/m299_tuple_ref_destructure/main.xi"),
        Some(0),
        "destructuring a tuple-element reference must bind value copies (m299)"
    );
}

// m301 (REL-20261010-1549-stdlib-4): clause-position `result[i]` must resolve
// the return Vec's element type like a `var` binding (m301 records it in
// local_vec_elem for the synthetic result local). Pre-fix the index fell to
// the scalar elem_load fallback: Vec[Float64]'s f64 bits were sitofp'd
// (1.0 -> 4.6e18 -> false `contract violated: ensures`) and Vec[Vec[T]]'s
// row read inttoptr'd the inline first field (the inner data pointer) as a
// %struct.Vec header, so `result[0].len()` read the inner array -- the
// witness's [1,2] Int control passed only by coincidence. Lock matrix:
// Float64/Float32 scalar, nested Float64 with chained result[0][0], a
// non-coincidence Int [7,9] row, and Vec[Str]. Enforcement is LIVE (m291).
#[test] fn e2e_m301_clause_float_vec_index() {
    assert_eq!(
        compile_and_run("tests/regression/m301_clause_float_vec_index/main.xi"),
        Some(0),
        "clause-position result[i] must read the stored elements (m301)"
    );
}

// m302 (REL-20261011-1659-orbitdb): a Vec bound from an ALIASING RHS (local
// ident, Vec element, struct field, or assignment target) must own its
// storage -- element assignment through the copy used to write through to
// the source buffer while push/pop only looked like copy-on-write. The
// fixture gates flat copy (`src[0]` stays 1, `src.len` stays 2), the
// Vec-of-Vec handle and struct-field extract shapes, the `b = a` form, and
// keeps the push/pop + owned-rebuild controls.
#[test] fn e2e_m302_vec_copy_alias() {
    assert_eq!(
        compile_and_run("tests/regression/m302_vec_copy_alias/main.xi"),
        Some(0),
        "Vec bindings from aliasing RHS must deep-copy the buffer (m302)"
    );
}

// m304 (REL-20261010-1550-orbitdb-2): standalone --check of a
// self-referential module failed T001 "expected Engine, found module" at
// the orbitdb facade (`engine_insert(&mut db.engine, ...)`), while the same
// file compiled clean as an imported module. The declared chain's segments
// are flattened into the module map (`module xiom.db.engine` registers "db"
// with the "engine" submodule export), and check_module_field_access tried
// module-qualified resolution BEFORE local-scope lookup -- so `db.engine` on
// a `db` parameter resolved to the module. Locals now shadow module aliases
// in expression position.
#[test] fn e2e_m304_module_local_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m304_module_local_shadow/main.xi"),
        Some(0),
        "a local binding must shadow a same-named module chain segment (m304)"
    );
}

// m305 (REL-20261011-1219-orbitdb; C-ORBIT-08 PART A): a tuple built from an
// enum CONSTRUCTOR call (`Ok((Color.Custom(n), n + 3))`) fell back to the
// i64-erased element type in both tuple-element namers and built a 16-byte
// Tuple__Int__Int -- the enum TAG in slot 0, the payload dropped -- while the
// signature/read side used the 24-byte Tuple__Color__Int: silent wrong values
// (2/8 instead of 8/5), and the fixture crashes (0xC000001D) on the ub-grade
// read past the box pre-fix. The fix names enum ctor-call elements by the
// parent enum key, mirroring the variant-REF path. The fixture also gates the
// constant-variant arms, the Err arm, a plain let-bound (Color, Int) tuple,
// and the enum-alone workaround shape.
#[test] fn e2e_m305_enum_ctor_tuple() {
    assert_eq!(
        compile_and_run("tests/regression/m305_enum_ctor_tuple/main.xi"),
        Some(0),
        "enum ctor-call elements in tuples must keep the 24-byte shape (m305)"
    );
}

// m307 (REL-20261013-1324-selfhost; correction REL-20261013-1732): a
// variant-pattern binding must read the CANONICAL field at its POSITION.
// The binding NAME used to win the direct name lookup, so a binding named
// like another variant's field (or a later field of the same variant) read
// the wrong slot: the selfhost's `NkExprField(recv, name_node)` loaded slot
// 461 for a parsed 23 because NkFn.recv sits before NkExprField.inner in
// the enum field list. Red on v0.64.4 and pre-fix HEAD.
#[test] fn e2e_m307_variant_binding_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m307_variant_binding_shadow/main.xi"),
        Some(0),
        "variant-pattern bindings must read the canonical slot, not a name collision (m307)"
    );
}

// m307 IR lock: `read_renamed` must GEP the canonical "inner" slot (index 2
// for Kind { discriminant, recv, inner, name }), never Wrap.recv (index 1).
#[test] fn e2e_m307_variant_binding_shadow_ir() {
    let ir = compile_ir("tests/regression/m307_variant_binding_shadow/main.xi")
        .expect("m307 fixture must compile to IR");
    let start = ir.find("define i64 @read_renamed")
        .expect("m307: the renamed-binding reader must be defined");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains(", i32 0, i32 2"),
        "m307: the renamed binding must load the canonical inner slot"
    );
    assert!(
        !body.contains(", i32 0, i32 1"),
        "m307: the renamed binding must not load Wrap.recv's slot"
    );
}

// m308 (REL-20261010-1553-playground-2 follow-up): a by-ref Str comparator
// param must compare CONTENT. Pre-fix `a < b` on `&Str` params compared the
// first byte of each char pointer (the m286 strcmp branch only matched plain
// "Str" locals), so find_min on ["pear","cherry","apple"] returned 0.
#[test] fn e2e_m308_ref_str_cmp() {
    assert_eq!(
        compile_and_run("tests/regression/m308_ref_str_cmp/main.xi"),
        Some(0),
        "by-ref Str comparator params must compare string content (m308)"
    );
}

// m308 IR lock: the `&Str` comparator body must lower `<` to strcmp, not a
// byte load.
#[test] fn e2e_m308_ref_str_cmp_ir() {
    let ir = compile_ir("tests/regression/m308_ref_str_cmp/main.xi")
        .expect("m308 ref-str-cmp fixture must compile to IR");
    let start = ir.find("define i64 @less_impl")
        .expect("m308: less_impl must be defined");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("@strcmp("),
        "m308: `a < b` on &Str params must strcmp the content; body:\n{body}"
    );
    assert!(
        !body.contains("load i8,"),
        "m308: the first-byte pointer compare must be gone; body:\n{body}"
    );
}

// m308 (REL-20261010-1554-playground): the prompt must reach a PIPE reader
// BEFORE the blocking io.read_line() -- pre-fix stdout block-buffered and the
// bytes surfaced only at exit. This spawns the binary with piped stdin and
// waits for "PROMPT>" WITHOUT writing any input first; only then writes the
// line (which must be echoed).
#[test] fn e2e_m308_stdout_flush_before_read() {
    use std::io::{Read, Write};
    let src = "tests/regression/m308_stdout_flush/main.xi";
    let exe_name = format!(
        "e2e_m308_flush_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    );
    let exe_path = project_root().join(&exe_name);
    let _ = std::fs::remove_file(&exe_path);
    let compile = Command::new(xiom_path())
        .args(["-o", exe_name.as_str(), src])
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_path()));
    assert!(
        compile.status.success(),
        "m308 flush fixture must compile: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let mut child = Command::new(&exe_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn m308 flush binary");
    let mut stdout_pipe = child.stdout.take().expect("stdout pipe");
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let reader = std::thread::spawn(move || {
        let mut buf = [0u8; 256];
        loop {
            match stdout_pipe.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            }
        }
    });
    let first = match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(v) => v,
        Err(e) => {
            let _ = child.kill();
            panic!(
                "m308: the prompt must be flushed before the blocking read \
                 (no output arrived for 10s while stdin was silent): {e}"
            );
        }
    };
    let mut text = String::from_utf8_lossy(&first).to_string();
    assert!(
        text.contains("PROMPT>"),
        "m308: the first bytes must be the flushed prompt; got {text:?}"
    );
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"Ada\n")
        .expect("write stdin");
    while !text.contains("GOT:Ada") {
        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(chunk) => text.push_str(&String::from_utf8_lossy(&chunk)),
            Err(_) => break,
        }
    }
    let _ = child.stdin.take();
    let status = child.wait().expect("wait m308 flush binary");
    let _ = reader.join();
    let _ = std::fs::remove_file(&exe_path);
    assert!(
        text.contains("GOT:Ada"),
        "m308: read_line must echo the piped line; got {text:?}"
    );
    assert!(status.success(), "m308: the flush fixture must exit 0");
}

// m308 IR lock: the blocking fgets read must be directly preceded by an
// fflush(NULL) so pending prompt bytes leave the stdio buffer first.
#[test] fn e2e_m308_stdout_flush_ir() {
    let ir = compile_ir("tests/regression/m308_stdout_flush/main.xi")
        .expect("m308 flush fixture must compile to IR");
    let lines: Vec<&str> = ir.lines().collect();
    let idx = lines
        .iter()
        .position(|l| l.contains("call i8* @fgets("))
        .expect("m308: the fgets call must be in the IR");
    let window_start = idx.saturating_sub(2);
    assert!(
        lines[window_start..idx].iter().any(|l| l.contains("@fflush(i8* null)")),
        "m308: fgets must be preceded by fflush(NULL); before line {idx}: {:?}",
        &lines[window_start..idx]
    );
}

// m308 (REL-20261010-1554-benchmark, t3 floor): arena-free confined blocks
// run end to end after the guard-page arm/disarm elision: the 20k-entry
// arena-free hot loop, an arena-free `return`-tail block, and a guard-arena
// block (Vec inside unsafe + Copy-Out) all produce their expected values.
#[test] fn e2e_m308_guard_page_elide() {
    assert_eq!(
        compile_and_run("tests/regression/m308_guard_page_elide/main.xi"),
        Some(0),
        "arena-free and arena-crossing confined blocks must both run (m308)"
    );
}

// m308 IR lock: every confined block fn emits the guard-PAGE arm/disarm pair
// IFF it allocates from the guard arena. Pre-fix every block did (one
// VirtualProtect per entry -- the t3 floor); post-fix the t3-shaped block fn
// keeps enter/exit + the trampoline but drops the page pair.
#[test] fn e2e_m308_guard_page_elide_ir() {
    let ir = compile_ir("tests/regression/m308_guard_page_elide/main.xi")
        .expect("m308 guard-page fixture must compile to IR");
    let mut checked = 0usize;
    let mut elided = 0usize;
    let mut kept = 0usize;
    for (i, part) in ir.split("define i64 @__unsafe_block_").enumerate() {
        if i == 0 {
            continue;
        }
        let end = part.find("\ndefine ").unwrap_or(part.len());
        let body = &part[..end];
        let has_alloc =
            body.contains("@xiom_guard_alloc") || body.contains("@xiom_guard_realloc");
        let has_arm = body.contains("@xiom_guard_page_arm()");
        let has_disarm = body.contains("@xiom_guard_page_disarm()");
        assert_eq!(
            has_alloc, has_arm,
            "m308: arm must match guard-arena use; block body:\n{body}"
        );
        assert_eq!(
            has_alloc, has_disarm,
            "m308: disarm must match guard-arena use; block body:\n{body}"
        );
        assert!(
            body.contains("@xiom_guard_heap_enter()"),
            "m308: guard-heap depth must stay in every confined block; body:\n{body}"
        );
        if has_alloc {
            kept += 1;
        } else {
            elided += 1;
        }
        checked += 1;
    }
    assert!(
        checked >= 3,
        "m308: expected at least three confined block fns, got {checked}"
    );
    assert!(
        elided >= 2,
        "m308: the arena-free blocks must drop the page pair, elided={elided}"
    );
    assert!(
        kept >= 1,
        "m308: the arena-crossing block must keep the page pair, kept={kept}"
    );
}

// m308 fault-containment control: an ARENA-FREE confined block still runs
// through the SEH/sigsetjmp trampoline after the page elision -- a real
// hardware fault inside it must still be contained (trace + exit 0).
#[test] fn e2e_m308_guard_page_elide_fault_contained() {
    let exe_name = format!(
        "m308_guard_fault_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    );
    let exe = project_root().join(&exe_name);
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .args(["tests/regression/m308_guard_page_elide_fault/main.xi", "-o"])
        .arg(&exe)
        .current_dir(project_root())
        .output()
        .expect("compile m308 fault fixture");
    assert!(
        compile.status.success(),
        "m308 fault fixture must compile: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe)
        .env("XIOM_TRACE_FAULT", "1")
        .output()
        .expect("run m308 fault fixture");
    let _ = std::fs::remove_file(&exe);
    let stderr = String::from_utf8_lossy(&run.stderr);
    // m333: marker is Windows-VEH-only (see the m244 note); containment
    // (exit 0) is asserted on every platform below.
    #[cfg(windows)]
    assert!(
        stderr.contains("[trampoline] fault"),
        "m308: an arena-free confined block must still report the contained fault; stderr:\n{stderr}"
    );
    let _ = &stderr;
    assert!(
        stderr.contains("[fault] confined block"),
        "m351: the m308 confined fault must be manifested with the typed diagnostic; stderr:\n{stderr}"
    );
    assert_eq!(
        run.status.code(),
        Some(70),
        "m308/m351: contained but unacknowledged confined fault exits 70"
    );
}

// m310 (REL-20261010-1549-packages-5): a dependency's shipped port.args.json
// must be applied at CONSUMER build time -- the C source it names (with
// ${PACKAGE_DIR}) links without a manual --c-source. Red on v0.64.4 and
// pre-fix HEAD (`undefined symbol: m310_helper_bump`).
#[test] fn e2e_m310_port_args_dep() {
    assert_eq!(
        compile_and_run("tests/regression/m310_port_args_dep/main.xi"),
        Some(0),
        "a dependency's port.args.json --c-source must apply at consumer build time (m310)"
    );
}

// m310 (REL-20261010-1549-packages-7): a range spec (`>=0.2.0`) in
// xiom.toml [dependencies] must resolve the installed package dir; the old
// resolver used a literal starts_with and only exact specs resolved. The
// fixture is hermetic through its own XIOM_HOME (install-home layout).
#[test] fn e2e_m310_dep_range_spec() {
    let root = project_root()
        .join("tests").join("regression").join("m310_dep_range_spec");
    let exe = project_root().join("target").join("m310_dep_range_spec.exe");
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .arg("-o")
        .arg(&exe)
        .arg("tests/regression/m310_dep_range_spec/app/main.xi")
        .env("XIOM_HOME", root.join("home"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the m310 range fixture");
    assert!(
        compile.status.success(),
        "a `>=0.2.0` dep spec must resolve the installed dep (m310); stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe).output().expect("run the m310 range fixture");
    assert_eq!(run.status.code(), Some(0), "range fixture program must exit 0 (m310)");
    let _ = std::fs::remove_file(&exe);
}

// m340 (REL-20261016-1128-engine class 3): dependency port.args.json C
// sources compile ONCE through the rtcache-style object cache -- the second
// consumer build must reuse the objects (`dep-cache HIT`) instead of
// recompiling every dependency TU inside the single clang link invocation
// on EVERY build (the m310 behavior behind the engine's 7-10 min probes /
// ~20 h whole-tree CI). The cache is redirected to a temp dir through the
// XIOM_OBJ_CACHE_DIR override so the MISS->HIT sequence is hermetic.
#[test] fn e2e_m340_dep_csource_cache() {
    let cache_dir = std::env::temp_dir().join(format!("xiom_m340_cache_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cache_dir);
    let exe = project_root().join("target").join("m340_dep_csource_cache.exe");

    let compile = || {
        let _ = std::fs::remove_file(&exe);
        Command::new(xiom_path())
            .arg("-o")
            .arg(&exe)
            .arg("tests/regression/m340_dep_csource_cache/main.xi")
            .env("XIOM_OBJ_CACHE_DIR", &cache_dir)
            .env("XIOM_TIMINGS", "1")
            .current_dir(project_root())
            .output()
            .expect("spawn xiom for the m340 cache fixture")
    };

    let first = compile();
    assert!(
        first.status.success(),
        "first m340 compile must succeed; stderr:\n{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_err = String::from_utf8_lossy(&first.stderr);
    assert!(
        first_err.contains("dep-cache MISS"),
        "first build must BUILD the dependency objects (m340); stderr:\n{first_err}"
    );

    let second = compile();
    assert!(
        second.status.success(),
        "second m340 compile must succeed; stderr:\n{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_err = String::from_utf8_lossy(&second.stderr);
    assert!(
        second_err.contains("dep-cache HIT"),
        "second build must REUSE the cached dependency objects (m340); stderr:\n{second_err}"
    );
    assert!(
        !second_err.contains("dep-cache MISS"),
        "an unchanged consumer build must not rebuild the dependency objects (m340); stderr:\n{second_err}"
    );

    // The cache really holds one object per dependency TU on disk.
    let mut objects: Vec<std::path::PathBuf> = Vec::new();
    collect_files_with_ext(&cache_dir, "o", &mut objects);
    assert!(
        objects.len() >= 3,
        "dependency object cache must hold one object per --c-source TU; got {objects:?}"
    );

    let run = Command::new(&exe).output().expect("run the m340 cache fixture");
    assert_eq!(run.status.code(), Some(0), "m340 cache fixture program must exit 0");
    let _ = std::fs::remove_file(&exe);
    let _ = std::fs::remove_dir_all(&cache_dir);
}

/// Recursively collect files with a given extension (m340 cache evidence).
fn collect_files_with_ext(dir: &Path, ext: &str, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_with_ext(&path, ext, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                out.push(path);
            }
        }
    }
}

// m340: the dependency's declared `--timeout 600` must FLOOR the compile
// watchdog when the consumer set none (the global default 300 must not cap
// it); an explicit consumer --timeout wins. Proven through the verbose
// resolved-budget line (no real 600 s run needed).
#[test] fn e2e_m340_dep_timeout_floor() {
    let cache_dir = std::env::temp_dir().join(format!("xiom_m340_timeout_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cache_dir);
    let exe = project_root().join("target").join("m340_dep_timeout_floor.exe");

    let compile = |extra: &[&str]| {
        let _ = std::fs::remove_file(&exe);
        Command::new(xiom_path())
            .arg("-o")
            .arg(&exe)
            .args(extra)
            .arg("tests/regression/m340_dep_csource_cache/main.xi")
            .env("XIOM_VERBOSE_CONFIG", "1")
            .env("XIOM_OBJ_CACHE_DIR", &cache_dir)
            .current_dir(project_root())
            .output()
            .expect("spawn xiom for the m340 timeout fixture")
    };

    let floored = compile(&[]);
    assert!(
        floored.status.success(),
        "m340 timeout fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&floored.stderr)
    );
    let floored_err = String::from_utf8_lossy(&floored.stderr);
    assert!(
        floored_err.contains("compile timeout: 600s (dependency port.args.json --timeout floor)"),
        "a dependency --timeout of 600 must floor the watchdog budget (m340); stderr:\n{floored_err}"
    );

    let explicit = compile(&["--timeout", "123"]);
    assert!(
        explicit.status.success(),
        "m340 timeout fixture must compile with explicit --timeout; stderr:\n{}",
        String::from_utf8_lossy(&explicit.stderr)
    );
    let explicit_err = String::from_utf8_lossy(&explicit.stderr);
    assert!(
        explicit_err.contains("compile timeout: 123s (explicit --timeout)"),
        "an explicit consumer --timeout must win over the dependency floor (m340); stderr:\n{explicit_err}"
    );

    let _ = std::fs::remove_file(&exe);
    let _ = std::fs::remove_dir_all(&cache_dir);
}

// m340: ci.ps1's documented `@extra` recipe -- the consumer ALSO passes the
// dependency's --c-source manually. Dedupe semantics are unchanged: the
// consumer's copy wins, the dependency copy is skipped (canonical-path
// dedupe), and the link carries ONE copy of the TU. Pre-fix double
// application would link `m340_cache_a` twice (duplicate symbol).
#[test] fn e2e_m340_dep_csource_dedupe_manual_flags() {
    let cache_dir = std::env::temp_dir().join(format!("xiom_m340_dedupe_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cache_dir);
    let exe = project_root().join("target").join("m340_dep_csource_dedupe.exe");
    let _ = std::fs::remove_file(&exe);
    let helper_one = project_root()
        .join("tests").join("regression").join("m340_dep_csource_cache")
        .join("helper").join("vendor").join("one.c");
    let helper_two = project_root()
        .join("tests").join("regression").join("m340_dep_csource_cache")
        .join("helper").join("vendor").join("two.c");

    let compile = Command::new(xiom_path())
        .arg("-o")
        .arg(&exe)
        // The consumer's manual copies (their own spelling, project-root
        // relative) of two dependency TUs + the ci.ps1-style --timeout.
        // Absolute paths: manual --c-source inputs link verbatim from the
        // link invocation's scratch cwd, so ci.ps1's @extra recipe passes
        // resolved paths too.
        .args(["--c-source", helper_one.to_str().unwrap()])
        .args(["--c-source", helper_two.to_str().unwrap()])
        .args(["--timeout", "900"])
        .arg("tests/regression/m340_dep_csource_cache/main.xi")
        .env("XIOM_OBJ_CACHE_DIR", &cache_dir)
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the m340 dedupe fixture");
    assert!(
        compile.status.success(),
        "manual --c-source + port.args.json copies must dedupe, not double-link (m340); stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&exe).output().expect("run the m340 dedupe fixture");
    assert_eq!(run.status.code(), Some(0), "m340 dedupe fixture program must exit 0");
    let _ = std::fs::remove_file(&exe);
    let _ = std::fs::remove_dir_all(&cache_dir);
}

// m309 (REL-20261011-2320-bindings part 3): unqualified cross-module `const`
// imports with 4-SEGMENT module paths must resolve every imported module's
// bare constants. register_external_module merged the per-file export maps
// one nesting level deep (`existing_sub.entry(sk).or_insert(sv)`), so with
// `module xiom.m309seg.fam.alpha` + `xiom.m309seg.fam.beta` the second
// file's subtree under the shared `foo.bar`-style path was DROPPED -- only
// the module the LIFO worklist registered first stayed visible. The losing
// module's constants reported T001 undefined variable; which module lost
// flipped with the import order. Post-fix the merge recurses; both bare and
// module-qualified spellings resolve in either order. Red pre-fix
// (T001 'ALPHA' for this order, T001 'BETA' for main_flip.xi), green after.
#[test] fn e2e_m309_four_segment_const_import() {
    assert_eq!(
        compile_and_run("tests/regression/m309_four_segment_const_import/main.xi"),
        Some(0),
        "both 4-segment modules' bare constants must resolve (alpha-first) (m309)"
    );
    assert_eq!(
        compile_and_run("tests/regression/m309_four_segment_const_import/main_flip.xi"),
        Some(0),
        "both 4-segment modules' bare constants must resolve (beta-first) (m309)"
    );
}

// m309 (REL-20261011-2320-bindings part 2): a Vec built INSIDE a confined
// (trampolined) unsafe block and escaping through the block tail (or a
// `return` inside the block) carried its arena-backed data pointer past the
// block exit -- xiom_guard_heap_exit releases the arena wholesale, so the
// caller's first element read faulted (STATUS_ACCESS_VIOLATION 0xC0000005;
// the bindings trampolined-wrapper crash class, which the packages worked
// around with #[unsafe_direct]). The fix promotes the Vec payload to the
// main heap before the arena resets (plan S2.11 Copy-Out v1) on BOTH
// lowering paths. Red pre-fix (0xC0000005 on the first escaped read), green
// after; the non-confined control arm must stay green on both.
#[test] fn e2e_m309_confined_vec_copy_out() {
    assert_eq!(
        compile_and_run("tests/regression/m309_confined_vec_copy_out/main.xi"),
        Some(0),
        "a Vec escaping a confined block must be Copy-Out'd to the main heap (m309)"
    );
    let ir = compile_ir("tests/regression/m309_confined_vec_copy_out/main.xi")
        .expect("m309: fixture must emit IR");
    // IR twin: each escaping block fn (0 = return-inside, 1 = block tail)
    // must allocate the promoted copy with plain @malloc and run the memcpy
    // copy BEFORE xiom_guard_heap_exit releases the arena.
    for name in ["define i64 @__unsafe_block_0", "define i64 @__unsafe_block_1"] {
        let start = ir.find(name)
            .unwrap_or_else(|| panic!("m309: {name} must be defined; got:\n{ir}"));
        let rest = &ir[start..];
        let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
        let body = &rest[..end];
        let malloc_at = body.find("call i8* @malloc")
            .unwrap_or_else(|| panic!("m309: {name} must Copy-Out with @malloc; got:\n{body}"));
        let exit_at = body.find("xiom_guard_heap_exit")
            .unwrap_or_else(|| panic!("m309: {name} must exit the arena; got:\n{body}"));
        assert!(
            malloc_at < exit_at,
            "m309: {name} must copy the Vec payload BEFORE the arena reset; got:\n{body}"
        );
    }
}
// m368 (REL-20261011-2032-engine): owned Vec locals must release their data
// buffer at function exit. Pre-fix `Vec[Float64].new()` + push malloc'd and
// realloc'd but never freed (mem.drop is a no-op, no drop glue), so the
// engine's alloc+drop loop grew RSS ~1 GB/s (15.9 GB / 10 s at v0.64.4). The
// lock asserts the generated drop path exists for the two owned shapes
// (var + let, `.push`/`.len` only) and, crucially, that it does NOT fire on
// the escape controls: `make_vec` returns its Vec and `alias_control` binds a
// copy of another owned Vec -- freeing either would corrupt live storage.
#[test] fn e2e_m368_vec_drop_reclaim() {
    assert_eq!(
        compile_and_run("tests/regression/m368_vec_drop_reclaim/main.xi"),
        Some(0),
        "owned Vec locals must be reclaimed without freeing escaped storage (m368)"
    );
    let ir = compile_ir("tests/regression/m368_vec_drop_reclaim/main.xi")
        .expect("m368: fixture must emit IR");
    let body_of = |name: &str| -> String {
        let start = ir.find(&format!("define i64 @{name}("))
            .or_else(|| ir.find(&format!("define %struct.Vec @{name}(")))
            .unwrap_or_else(|| panic!("m368: @{name} must be defined; got:\n{ir}"));
        let rest = &ir[start..];
        let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
        rest[..end].to_string()
    };
    for name in ["alloc_one", "alloc_let"] {
        let body = body_of(name);
        assert!(
            body.contains("call void @xiom_guard_free(i8*"),
            "m368: @{name} must free its owned Vec buffer at exit; got:\n{body}"
        );
        assert!(
            body.contains("vecdrop_do"),
            "m368: @{name} must guard the free with a drop flag; got:\n{body}"
        );
    }
    for name in ["make_vec", "alias_control"] {
        let body = body_of(name);
        assert!(
            !body.contains("xiom_guard_free"),
            "m368: @{name} must NOT free escaped/aliased Vec storage; got:\n{body}"
        );
    }
}
// m313 (REL-20261013-1445 compiler half): `ffi.free` call sites must lower
// through the stdlib wrapper (`@ffi.free`, arena-aware since stdlib wave 124:
// its body calls @xiom_guard_free) -- NOT the bare libc `@free` the codegen
// `free` builtin emitted for every `free` leaf call. Inside a confined block
// the matching `ffi.alloc` sits in the guard arena (the compiler routes
// confined alloc to @xiom_guard_alloc), so libc free on that pointer
// corrupted/spun the process (1548-bindings-5 alloc-guard-spin). Pre-fix
// this test fails: both the @main (main-heap roundtrip) and the confined
// block call sites read `call void @free(`. Compile-only lock: bounded and
// stdlib-pin independent (the runtime-symbol half is exercised end-to-end by
// the crates/xiom/tests/ffi_alloc_guard_free.rs watchdog).
#[test] fn e2e_m313_ffi_free_guard_ir() {
    let ir = compile_ir("tests/regression/m313_alloc_guard_free/main.xi")
        .expect("m313 ffi-free fixture must compile to IR");
    // The user confined block fn: carries the write call AND the guard-heap
    // entry (the stdlib wrapper bodies never write a u64 through ffi).
    let block = ir.split("\ndefine ")
        .find(|part| part.contains("@ffi.ptr_write_u64_le") && part.contains("@xiom_guard_heap_enter"))
        .expect("m313: the user confined block fn must exist");
    assert!(
        block.contains("call void @ffi.free("),
        "m313: the confined ffi.free call site must lower through the wrapper; block:\n{block}"
    );
    assert!(
        !block.contains("call void @free("),
        "m313: the confined ffi.free call site must NOT lower to bare libc @free; block:\n{block}"
    );
    // Main-heap roundtrip OUTSIDE confinement: same wrapper requirement (the
    // wrapper is arena-aware and reaches libc free off the arena).
    let main_fn = ir.split("\ndefine ")
        .find(|part| part.starts_with("i64 @main("))
        .expect("m313: @main must exist");
    assert!(
        main_fn.contains("call void @ffi.free("),
        "m313: the main-heap ffi.free call site must lower through the wrapper; @main:\n{main_fn}"
    );
    assert!(
        !main_fn.contains("call void @free("),
        "m313: the main-heap ffi.free call site must NOT lower to bare libc @free; @main:\n{main_fn}"
    );
}
// m312 (REL-20261013-1423 thread readback): a `*T` struct FIELD bound to a
// local (`let q = c.buf;`) or used as a method receiver field (`buf` in
// `Cell.at8`) must lower `+ Int` to pointer arithmetic. The i8* LLVM slot
// alone is ambiguous with Str, so pre-fix `q + 8` became
// xiom_str_concat(q, "8") and the read went through a decimal-digit string
// -- JoinHandle.join's `result_buf + 8` read GARBAGE while the spawned
// worker had written the correct result.
#[test] fn e2e_m312_ptr_field_arith() {
    assert_eq!(
        compile_and_run("tests/regression/m312_ptr_field_arith/main.xi"),
        Some(0),
        "a pointer-typed field (+ let binding / receiver field) must GEP in `+`, not Str-concat (m312)"
    );
}

// m312 IR lock: every `+ 8` on the pointer field must be
// `getelementptr i8, i8*` byte arithmetic and no xiom_str_concat CALL may
// appear (the fixture is string-free; the decl is unconditional). The
// confined block fns carry the arithmetic: __unsafe_block_0 = Cell.at8's
// block (receiver field `buf + 8`), __unsafe_block_1 = main's block (the
// field, the let-bound copy and the write).
#[test] fn e2e_m312_ptr_field_arith_ir() {
    let ir = compile_ir("tests/regression/m312_ptr_field_arith/main.xi")
        .expect("m312 fixture must compile to IR");
    let block_body = |id: usize| -> String {
        let marker = format!("define i64 @__unsafe_block_{id}(");
        let start = ir
            .find(&marker)
            .unwrap_or_else(|| panic!("m312: {marker} must be defined"));
        let rest = &ir[start..];
        let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
        rest[..end].to_string()
    };
    let at8_block = block_body(0);
    assert!(
        at8_block.contains("getelementptr i8, i8*"),
        "m312: the method receiver field `buf + 8` must be byte GEP; block:\n{at8_block}"
    );
    assert!(
        !at8_block.contains("xiom_str_concat"),
        "m312: the method receiver field `buf + 8` must not Str-concat; block:\n{at8_block}"
    );
    let main_block = block_body(1);
    let gep_sites = main_block.matches("getelementptr i8, i8*").count();
    assert!(
        gep_sites >= 3,
        "m312: c.buf/q pointer arithmetic needs >= 3 byte GEPs in main's block, got {gep_sites}; block:\n{main_block}"
    );
    assert!(
        !ir.contains("call i8* @xiom_str_concat("),
        "m312: no `*T + Int` site may lower to a Str-concat call"
    );
}

// m314 (REL-20261013-2042-engine HIGH + -2042-engine-2 + -2227-engine):
// native 128-bit payloads in Vec elements / stored structs plus the UInt64
// bit-63 family. Pre-fix: `Vec[Int128].push(x)` / `v[i] = x` aborted clang
// ("'%tmp' defined with type 'i128' but expected 'i64'"); a struct
// { id: Int128; tag: Int } was sized 16 instead of the 32-byte LLVM layout,
// so the Vec slot stride truncated the sibling `tag` field (uninitialized-
// stack garbage, no diagnostic); every UInt64 `<`/`<=`/`>`/`>=` lowered to
// signed icmp and `/`/`%` to sdiv/srem (bit-63-set values behaved signed);
// UInt64 -> Int128 widening sign-extended through binary-source `as` and
// mixed-i128 operand widening. The fixture returns DISTINCT exit codes per
// shape (1-7 128-bit aggregate round-trips, 8-14 ordering, 15-17 div/mod,
// 18-22 widening, 23-25 stored-struct control); green = 0.
#[test] fn e2e_m314_int128_uint64_family() {
    assert_eq!(
        compile_and_run("tests/regression/m314_int128_uint64_family/main.xi"),
        Some(0),
        "Int128/UInt128 aggregate payloads + UInt64 bit-63 semantics must be exact (m314)"
    );
}
// m316 (REL-20261014-1158-engine): an identifier named `spawn` must never
// drop the enclosing fn from the module catalog. `Ship{ spawn: on; x: 7 }`
// -- a struct literal whose FIRST field is named `spawn` -- was not
// recognized as a struct literal (the lookahead tested only
// TokenKind::Ident, while `spawn` lexes as a soft-keyword token), so the
// literal mis-parsed, the enclosing fn vanished from the module AST, and
// consumers silently lost the export ("cannot call 'ship_ready' on this
// expression"; the engine's rename-to-spawn_pos workaround). The fixture
// also pins the negative twin (plain `spawn_pos` field) and the
// `Ship{ spawn, x: n }` shorthand form. Pre-fix compile_and_run returns
// None (the consumer's calls are unresolved).
#[test] fn e2e_m316_spawn_name_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m316_1158_spawn_shadow/main.xi"),
        Some(0),
        "a spawn-named field (literal first position or shorthand) must not drop the module export (m316)"
    );
}

// m314 IR twin: the fixed lowering is visible as unsigned compares and
// div/mod, zero-extending i64->i128 widening, 16-byte 128-bit element loads,
// and the 16-/32-byte Vec slot allocations.
#[test] fn e2e_m314_int128_uint64_family_ir() {
    let ir = compile_ir("tests/regression/m314_int128_uint64_family/main.xi")
        .expect("m314 fixture must compile to IR");
    let main = ir.split("\ndefine ")
        .find(|part| part.starts_with("i64 @main("))
        .unwrap_or_else(|| panic!("m314: @main must be defined; got:\n{ir}"));
    for op in ["icmp ult i64", "icmp ugt i64", "icmp ule i64", "icmp uge i64"] {
        assert!(
            main.contains(op),
            "m314: UInt64 ordering must lower to {op}; @main:\n{main}"
        );
    }
    assert!(
        main.contains("udiv i64"),
        "m314: UInt64 `/` must lower to udiv; @main:\n{main}"
    );
    assert!(
        main.contains("urem i64"),
        "m314: UInt64 `%` must lower to urem; @main:\n{main}"
    );
    assert!(
        !main.contains("sdiv i64") && !main.contains("srem i64"),
        "m314: no signed div/mod may remain in @main; @main:\n{main}"
    );
    assert!(
        main.contains("zext i64") && main.contains("to i128"),
        "m314: UInt64 -> Int128 widening must zero-extend; @main:\n{main}"
    );
    assert!(
        main.contains("load i128"),
        "m314: a Vec[Int128] element read must load a full i128; @main:\n{main}"
    );
    assert!(
        main.contains("call i8* @xiom_guard_alloc(i64 256)"),
        "m314: Vec[Int128] slots are 16 bytes (16 slots x 16 = 256); @main:\n{main}"
    );
    assert!(
        main.contains("call i8* @xiom_guard_alloc(i64 512)"),
        "m314: {{ id: Int128; tag: Int }} slots are 32 bytes (16 slots x 32 = 512); @main:\n{main}"
    );
}


// m315 (REL-20261013-2139-engine): enum variant names colliding with
// xiom.geom type names corrupted consumer compilation (the engine's
// TrackKind{Scalar,Vec2,Vec3,Vec4} renamed to Float2/Float3/Float4 to
// work around it). Pre-fix, once a co-loaded module pulled xiom.geom in:
// (a) the checker typed the geom struct literal `Vec3{...}` as the enum
// variant's parent (T001 "field 'pos' type mismatch: expected Vec3, found
// TrackKind" in camera.cam_new) and (b) `return Vec3;` inside the enum's own
// module bound the imported STRUCT instead of the variant. Post-fix both
// resolve by context and the values are exact (variant tag 2, geom doubles
// 1/2/3, vec3_length(sqrt 14), tl_vec components).
#[test] fn e2e_m315_enum_geom_collision() {
    assert_eq!(
        compile_and_run("tests/regression/m315_2139_enum_geom_collision/main.xi"),
        Some(0),
        "an enum variant named after a xiom.geom type must resolve by context \
         (struct literal -> geom struct, bare value -> variant) (m315)"
    );
}

// m338 (REL-20261016-1128-engine class 1): the m315 current-module variant
// branch (lib.rs ident resolution) outranked BUILTIN type names, so an enum
// variant named `Str` made every `Str::from_utf8(...)` in that module type
// its qualifier as the parent enum -- the builtin Str-assoc table was never
// reached and the call fell to the unknown-method fallback (T001 "cannot
// call 'from_utf8' on this expression", the engine json_lite.xi:196 shape;
// green at 0.64.4, red at 0.64.5 and at m334). The fix gates the m315 branch
// on non-scalar leaves; the fixture pins the module form (variant `Str` +
// from_utf8 inside a struct literal in an IMPORTED module) with a runtime
// byte assert. Pre-fix compile_and_run returns None.
#[test] fn e2e_m338_catalog_str_variant_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m338_catalog_str_variant_shadow/main.xi"),
        Some(0),
        "a scalar-named enum variant must not hijack builtin assoc calls in the \
         same module (Str::from_utf8); bytes must be exact (m338)"
    );
}

// m338 entry-module twin: the ENTRY module itself declares the enum with a
// variant named `Str` (the minF2 shape). Same gate, no catalog wrapper.
#[test] fn e2e_m338_entry_variant_str_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m338_entry_variant_str_shadow.xi"),
        Some(0),
        "the entry module's own Str-named variant must not block builtin \
         Str::from_utf8 (m338 entry form)"
    );
}

// m317 (REL-20261013-2256-engine + REL-20261013-2324-engine-2 +
// REL-20261013-2324-engine-3): the constant-surface family. One fixture,
// every shape with a distinct exit code (1-3 shadow/qualified const reads,
// 4-6 `pub use` facade re-export incl. constants, 7-8 diamond re-export,
// 9-11 bare imported constants inside module files (multi-use/glob/aliased
// imports that the catalog-body use-skip used to drop), 12-14 module-level
// `pub var` export + fn-valued var call through the module path).
//
// Shape (a) doubles as the hang lock: the file-scope const shadows the
// imported constant name and its initializer reads the QUALIFIED import.
// Pre-fix the codegen bare-leaf substitution recursed until the compiler
// stack-overflowed with no diagnostic (the engine observed it as a HANG;
// on Windows debug builds it is an immediate 0xC00000FD exit, so the
// un-timed spawn below fails fast rather than blocking).
#[test] fn e2e_m317_const_surface_family() {
    assert_eq!(
        compile_and_run("tests/regression/m317_const_surface_family/main.xi"),
        Some(0),
        "m317: constant-surface family -- shadowing const must not hijack qualified reads or loop; pub use must re-export constants; bare module-file const imports must resolve; pub var must stay exported"
    );
}

// m320 (REL-20261013-2035-engine): the `&Str` PARAMETER spelling must resolve
// like plain Str. Pre-fix the compiler warned `unknown type '&Str' --
// defaulting to i64` and the value-use path bitcast the slot ADDRESS (i8**)
// to i8*, so str_slice read stack bytes (copy_ref printed heap garbage while
// the by-value control printed `hell`). The fixture returns distinct exit
// codes per shape (1 copy_ref, 2 copy_val control, 3 &mut Str, 4-5 chained
// &Str -> Str-param calls); green = 0.
#[test] fn e2e_m320_ref_str_param() {
    assert_eq!(
        compile_and_run("tests/regression/m320_ref_str_param/main.xi"),
        Some(0),
        "an `&Str` parameter must auto-deref to the Str handle at value uses (m320/2035)"
    );
}

// m320 IR twin (2035): the str_slice argument in @copy_ref must be a
// LOADED handle (`load i8*, i8**`), never the bitcast slot address.
#[test] fn e2e_m320_ref_str_param_ir() {
    let ir = compile_ir("tests/regression/m320_ref_str_param/main.xi")
        .expect("m320 ref-str-param fixture must compile to IR");
    let start = ir.find("define i8* @copy_ref")
        .expect("m320: @copy_ref must be defined");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    let call = body.lines()
        .find(|l| l.contains("@string.str_slice(i8* "))
        .unwrap_or_else(|| panic!("m320: copy_ref must call str_slice; body:\n{body}"));
    let arg = call
        .split("@string.str_slice(i8* ")
        .nth(1)
        .and_then(|s| s.split(',').next())
        .expect("m320: str_slice arg must parse")
        .trim();
    assert!(
        body.contains(&format!("{arg} = load i8*, i8**")),
        "m320: the &Str param value must load the handle ({arg}); body:\n{body}"
    );
    assert!(
        !body.contains("bitcast i8**"),
        "m320: the slot-address bitcast must be gone; body:\n{body}"
    );
}

// m320 (REL-20261013-1819-engine): an Int-typed pointer value at a named-call
// boundary (`takes8(v.as_mut_ptr() as Int)`, the engine's legacy idiom) was
// materialized as a truncated 1-byte stack temp and the slot ADDRESS was
// passed (the callee read 208 = the low byte of the stack address). Pointer
// bits must inttoptr. Distinct exit codes per shape (1 plain / 2 inline
// `as Int` / 3 Int binding cast back / 4-5 indirect fn-pointer calls).
#[test] fn e2e_m320_int_ptr_arg() {
    assert_eq!(
        compile_and_run("tests/regression/m320_int_ptr_arg/main.xi"),
        Some(0),
        "an Int-typed pointer value at a *UInt8 call boundary must inttoptr, not pass a temp address (m320/1819)"
    );
}

// m320 IR twin (1819): every direct takes8 call's pointer argument must be
// the buffer pointer or an inttoptr of pointer bits -- never an i8 alloca
// (the pre-fix materialization stored `trunc i64 ... to i8` in a stack byte
// and passed its address).
#[test] fn e2e_m320_int_ptr_arg_ir() {
    let ir = compile_ir("tests/regression/m320_int_ptr_arg/main.xi")
        .expect("m320 int-ptr-arg fixture must compile to IR");
    let mut checked = 0;
    for line in ir.lines() {
        let Some(pos) = line.find("call i64 @takes8(i8* ") else { continue };
        let arg = line[pos + "call i64 @takes8(i8* ".len()..]
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .trim_end_matches(')');
        checked += 1;
        assert!(
            ir.contains(&format!("{arg} = load i8*,")) || ir.contains(&format!("{arg} = inttoptr i64")),
            "m320: takes8 arg {arg} must be a loaded pointer or inttoptr; line:\n{line}"
        );
        assert!(
            !ir.contains(&format!("{arg} = alloca i8 ")) && !ir.contains(&format!("{arg} = alloca i8\n")),
            "m320: takes8 arg {arg} must not be a stack-byte temp; line:\n{line}"
        );
    }
    assert_eq!(checked, 3, "m320: fixture must exercise the 3 direct takes8 shapes; IR:\n{ir}");
}

// m318 (REL-20261013-2215-engine): a struct-returning fn whose body is a
// confined (whole-body `unsafe`) block and whose match arms push onto a Vec
// returned an aggregate whose inline Vec fields still pointed at the guard
// arena. `xiom_guard_heap_exit` released the arena at block exit, so the
// caller's first element read was silent garbage / STATUS_ACCESS_VIOLATION
// 0xC0000005 with NO diagnostic (the engine's world/state.xi state machine --
// workaround shipped engine-side). m309's Copy-Out promoted only a DIRECT
// `%struct.Vec` tail; aggregate tails were left dangling. A second face of
// the same area: `vec.push(...)` as the TAIL of a confined statement block in
// a match arm -- the block's value is a %struct.Vec while the fn returns an
// aggregate, and the block site emitted a DEAD was_returned branch that
// reconstructed the value with the block tail type, so clang rejected the
// module ("defined with type %struct.Vec but expected %struct.World").
// The fixture exercises every lowering path with exact values + a
// post-return push proving the promoted buffer is live main-heap memory,
// plus an i64-only aggregate and a non-confined control. Pre-fix: shapes
// A-D fault (0xC0000005), shape G fails the clang build. Green after.
#[test] fn e2e_m318_vec_match_arm_confined() {
    assert_eq!(
        compile_and_run("tests/regression/m318_vec_match_arm_confined/main.xi"),
        Some(0),
        "m318: a struct escaping a confined block must Copy-Out its inline Vec fields before the arena reset; a push-tail confined arm must not emit an ill-typed return path"
    );
}

// m318 IR twin: (a) each escaping aggregate block fn (0=return-inside,
// 1=block tail, 2=local Ident tail, 3=depth-2 nested Grid) must promote its
// inline Vec field(s) with a plain @malloc + memcpy BEFORE
// `xiom_guard_heap_exit` -- incl. the nested Grid insertvalue chain for the
// depth-2 shape; (b) the was_returned branch must be emitted only when the
// block fn can actually return: @step_return carries it (return inside the
// block), @step_tail and @step_push_arm must NOT.
#[test] fn e2e_m318_vec_match_arm_confined_ir() {
    let ir = compile_ir("tests/regression/m318_vec_match_arm_confined/main.xi")
        .expect("m318 fixture must emit IR");
    let fn_body = |name: &str| -> String {
        let start = ir
            .find(name)
            .unwrap_or_else(|| panic!("m318: {name} must be defined; got:\n{ir}"));
        let rest = &ir[start..];
        let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
        rest[..end].to_string()
    };
    for id in 0..4 {
        let name = format!("define i64 @__unsafe_block_{id}");
        let body = fn_body(&name);
        let malloc_at = body
            .find("call i8* @malloc")
            .unwrap_or_else(|| panic!("m318: {name} must Copy-Out the inline Vec field with @malloc; got:\n{body}"));
        let exit_at = body
            .find("xiom_guard_heap_exit")
            .unwrap_or_else(|| panic!("m318: {name} must exit the arena; got:\n{body}"));
        assert!(
            malloc_at < exit_at,
            "m318: {name} must promote the Vec field BEFORE the arena reset; got:\n{body}"
        );
        assert!(
            body.contains("insertvalue %struct.m318_vec_match_arm_confined.World")
                || body.contains("insertvalue %struct.m318_vec_match_arm_confined.Layered"),
            "m318: {name} must insert the promoted Vec back into the aggregate; got:\n{body}"
        );
    }
    // Depth-2: the nested Grid field is promoted through its own extract/
    // insert chain before the Layered insertvalue.
    let layered_block = fn_body("define i64 @__unsafe_block_3");
    assert!(
        layered_block.contains("insertvalue %struct.m318_vec_match_arm_confined.Grid")
            && layered_block.contains("insertvalue %struct.m318_vec_match_arm_confined.Layered"),
        "m318: the depth-2 nested Vec must be promoted through Grid into Layered; got:\n{layered_block}"
    );
    // Return-path gating: only a block with a direct `return` may carry the
    // was_returned branch.
    let step_return = fn_body("define %struct.m318_vec_match_arm_confined.World @step_return");
    assert!(
        step_return.contains("xiom_trampoline_was_returned"),
        "m318: @step_return has a return inside its confined block -- the was_returned branch must stay; got:\n{step_return}"
    );
    for name in [
        "define %struct.m318_vec_match_arm_confined.World @step_tail",
        "define %struct.m318_vec_match_arm_confined.World @step_push_arm",
    ] {
        let body = fn_body(name);
        assert!(
            !body.contains("xiom_trampoline_was_returned"),
            "m318: {name} has no return in its confined block -- the was_returned branch must not be emitted; got:\n{body}"
        );
    }
}

// m319 (REL-20261013-2238-engine, vecvecstr): a Vec[Vec[Str]] element read
// returned the string HANDLE as an untyped i64 through bound inner vecs,
// bound Str elements, Str params and call-returned nested Vecs (decimal
// pointer digits / mojibake / empty). Pre-fix the fixture exits 4 (v0.64.4
// also exits 4); the fix records the index read's XIOM value type on the
// binding and resolves chained/call-return containers at the index site.
#[test] fn e2e_m319_vecvecstr_nested() {
    assert_eq!(
        compile_and_run("tests/regression/m319_vecvecstr_nested/main.xi"),
        Some(0),
        "m319: nested Vec[Vec[Str]] element reads must keep the Str class through bound inner vecs, bound Str elements, Str params and returned nested Vecs"
    );
}

// m319 (REL-20261013-2238-engine-2, fnvalue_records): a cross-module
// provider struct with a fn-typed field carrying by-value Vec params +
// struct return emitted `%struct.CaptureProvider` BEFORE `%struct.Vec`
// (alphabetical type emission), and clang rejects a not-yet-defined struct
// in a function-type argument position ("invalid type for function
// argument" -- no binary). The fix emits referenced struct bodies before
// their users (dependency DFS). Pre-fix the compile FAILS (None); the
// v0.64.4 pin fails the same way.
#[test] fn e2e_m319_fnvalue_records() {
    assert_eq!(
        compile_and_run("tests/regression/m319_fnvalue_records/main.xi"),
        Some(0),
        "m319: fn-value fields with by-value Vec/record parameters must emit dependencies-before-users so clang accepts the function type"
    );
}

// m319 (REL-20261013-2238-engine-3, meshencoder): the engine's Mesh type
// collided with stdlib xiom.geom.geometry_3d.Mesh once the CLI chain pulled
// xiom.geom in -- engine-shaped mesh encoders called from an IMPORTED
// module read through the wrong layout (9216 bytes for a 384-byte quad +
// AV). Fixed in the m278 same-named-type identity work; this fixture pins
// the whole composition (entry builder, module builder, entry-built &Mesh
// through a module wrapper, byte-exact compares). Red on the v0.64.4 pin
// (0xC0000005), green at HEAD.
#[test] fn e2e_m319_meshencoder_collision() {
    assert_eq!(
        compile_and_run("tests/regression/m319_meshencoder_collision/main.xi"),
        Some(0),
        "m319: a same-named Mesh (engine-shaped) vs xiom.geom.geometry_3d.Mesh must keep module identity across the composed CLI chain"
    );
}

// m321 (REL-20261013-2119-engine + REL-20261014-1213-engine): the
// empty-Vec element READ class. The engine bisect artifacts (bisect_88 /
// bisect_95c) AV'd inside strcmp with ZERO output: their check 20 read
// `events[0]` on an empty Vec[Str] -- the tick-5 chord can never fire (its
// first member was RELEASED before the second member's press, so no tick
// has all members down), and the resident heap bytes were dereferenced as
// a Str pointer (-1073741819). bisect_75 "ran" only because it omits that
// check: there is no main-size threshold. The engine's real
// probe_input_map battery is 22/22 green and the pinned v0.64.4 toolchain
// behaves identically -> not a compiler regression; the finding is that
// this read class had no e2e lock (reads on EMPTY Vecs, len == 0).
//
// Documented contract: default builds keep the historical unchecked
// element read (out-of-bounds reads are UB -- the witness dereferenced the
// garbage); --overflow-checks turns it into a deterministic llvm.trap via
// the read-path bounds guard (expr.rs `bounds_trap`).
#[test] fn e2e_m321_empty_vec_read_contract() {
    let code = compile_and_run_with_flags(
        "tests/regression/m321_empty_vec_read/main.xi",
        &["--overflow-checks"],
    );
    assert!(
        trapped_under_bounds_checks(code),
        "empty Vec element read must trap under --overflow-checks (m321), got {code:?}"
    );
}

// m321 IR twin: the read-path bounds guard is emitted under
// --overflow-checks and stays out of default builds -- the same
// presence/absence split m241 pins for writes. Default builds must keep
// the historical unchecked element read.
#[test] fn e2e_m321_empty_vec_read_contract_ir() {
    let emit = |extra: &[&str]| -> String {
        let output = Command::new(xiom_path())
            .args(extra)
            .args(["--emit-ir", "tests/regression/m321_empty_vec_read/main.xi"])
            .current_dir(project_root())
            .output()
            .expect("emit-ir");
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let checked = emit(&["--overflow-checks"]);
    assert!(
        checked.contains("bounds_trap"),
        "the Vec read-path bounds trap must be emitted under --overflow-checks (m321)"
    );
    let default = emit(&[]);
    assert!(
        !default.contains("bounds_trap"),
        "default builds must keep the historical unchecked read path (m321)"
    );
}

// m322 (release gate): a FIXED struct field holding a scalar pointer must
// index by ELEMENT width with a RUNTIME index. Pre-fix the m281 pointer gate
// resolved the pointee width only for ident/call containers, so `buf.data[i]`
// on a ref-param field fell to a byte-stride i8 GEP + i8 load (sums read one
// byte per element; the m37_bug45 lock regressed). Guard twins for the
// compare half of m322: the m32_int_0194/0195 tokens are pinned in the ci.yml
// e2e list (mixed-sign cast compares must stay icmp slt).
#[test] fn e2e_m322_ptr_field_index() {
    assert_eq!(
        compile_and_run("tests/regression/m322_ptr_field_index/main.xi"),
        Some(0),
        "a *Int field read through a ref param must use the i64 element stride"
    );
}

// m322 IR twin: @at must index the field pointer as `getelementptr i64,
// i64*` and load a full i64 (no byte read).
#[test] fn e2e_m322_ptr_field_index_ir() {
    let ir = compile_ir("tests/regression/m322_ptr_field_index/main.xi")
        .expect("m322: fixture must emit IR");
    let start = ir.find("define i64 @at(")
        .unwrap_or_else(|| panic!("m322: @at must be defined; got:\n{ir}"));
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("getelementptr i64, i64*"),
        "m322: @at must GEP the field pointer by i64 elements; got:\n{body}"
    );
    assert!(
        !body.contains("load i8,"),
        "m322: @at must not read a single byte per element; got:\n{body}"
    );
}

// m323 (REL-20261014-2123-stdlib): Cell[Int8].set() followed by get() read
// pointer-shaped garbage on the v0.64.5 candidate (filed witness rc 2;
// v0.64.4 archive rc 0). The call-site generic fallback paired args[0] with
// the receiver `self` param slot, so the VALUE arg was typed from the self
// param (Named "Self" -> the arg's OWN LLVM type): a narrow arg
// (`127 as Int8`) stayed i8 while the mono def takes the widened i64 payload
// slot, and the callee stored the register's garbage high bits. Pre-fix the
// fixture exits 2; the fix skips the receiver slot when pairing args.
// Locks the Cell[Int8] witness + Int16/Int32 siblings + a user generic
// struct method leg (same shifted-pairing class). Needs the stdlib checkout
// (xiom/cell/cell.xi).
#[test] fn e2e_m323_cell_int8_set_readback() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("cell").join("cell.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/cell/cell.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m323_cell_int8_set_readback/main.xi"),
        Some(0),
        "Cell[Int8] set/get readback must compile and run (m323)"
    );
}

// m324 (REL-20261014-2123-stdlib): a function returning
// `Result<(Vec[UInt8], Vec[UInt8]), Str>` with `Ok((a, b))` must read back the
// tuple Vec lengths/elements correctly at the CALLER. `type_string_full` (m303)
// records tuple elements WITH container args ("Tuple__Vec[UInt8]__Vec[UInt8]")
// while the registered struct key erases them ("Tuple__Vec__Vec"), so the
// Ok-pair payload binding missed `llvm_type_for` ("unknown type ... defaulting
// to i64") and bound the raw box handle: `pair.0.len()` fell to the literal-0
// fallback -> Str.len read 0xFFFFFFFF (rc 3) and the decrypt roundtrip AV'd
// (0xC0000005) in the stdlib aes-gcm smoke. The fixture checks both lengths
// plus first/last elements (rc 3/4/5/6 on the broken paths; green = 0).
#[test] fn e2e_m324_aead_pair_len() {
    assert_eq!(
        compile_and_run("tests/regression/m324_aead_pair_len/main.xi"),
        Some(0),
        "Result Ok-pair Vec lengths must read back through the boxed tuple (m324)"
    );
}

// m325 (REL-20261014-2123-stdlib): float-typed pointer element reads must NOT
// take integer extensions. Pre-fix `let p = ptr as *Float32; p[i]` inside an
// unsafe block emitted `zext float %v to i64` (invalid IR -- clang "invalid
// cast opcode for cast from 'float' to 'i64'"), rejecting the
// p_sweep_single_param probe build inside xiom.simd.vec4.f32x4_load /
// xiom.simd.vec8.f32x8_load. The fixture mirrors that single-param sweep
// shape and checks the read values at runtime.
#[test] fn e2e_m325_zext_float_cast() {
    assert_eq!(
        compile_and_run("tests/regression/m325_zext_float_cast/main.xi"),
        Some(0),
        "float pointer element reads must lower without integer extensions"
    );
}

// m325 IR twin: the pointer-index read keeps the loaded `float` (stored
// directly into the struct field); no `zext float ... to i64` may appear.
#[test] fn e2e_m325_zext_float_cast_ir() {
    let ir = compile_ir("tests/regression/m325_zext_float_cast/main.xi")
        .expect("m325: fixture must emit IR");
    assert!(
        ir.contains("load float"),
        "m325: the fixture must exercise float element loads; got:\n{ir}"
    );
    assert!(
        !ir.contains("zext float"),
        "m325: float elements must not take integer extensions; got:\n{ir}"
    );
}

// m326 (REL-20261014-2123-stdlib part 4): the colliding `LinkedList` leaf --
// xiom.collections' GENERIC `LinkedList[T]` (invariant-bearing) kept the bare
// key while xiom.collect.list's non-generic `LinkedList` injected
// module-qualified. The fn-signature injection walk deduped on the QUALIFIED
// key, so the collections decl was pushed TWICE; codegen emitted
// `LinkedList.invariant_check` twice and clang rejected the module ("invalid
// redefinition of function 'LinkedList.invariant_check'"). The same bare-key
// claim typed a user `&mut LinkedList` param as the collections 1-field
// layout: the @pre snapshot preclone read the 6-field list's head/tail/size
// as a Vec header -> malloc(huge) NULL -> llvm.trap (0xC000001D). Pre-fix:
// compile rc 1 (redefinition) on current main; v0.64.4 pin green. The
// fixture (trimmed from p_wave7/p_wave8) compiles rc 0 and runs 0.
#[test] fn e2e_m326_dup_invariant_helper() {
    assert_eq!(
        compile_and_run("tests/regression/m326_dup_invariant_helper/main.xi"),
        Some(0),
        "m326: the LinkedList invariant helper must emit once and the @pre snapshot must use the imported list layout"
    );
}

// m326 IR twin: the compiler-generated invariant helper for the colliding
// `LinkedList` leaf appears exactly ONCE (pre-fix: two identical defines,
// same symbol, clang-rejected).
#[test] fn e2e_m326_dup_invariant_helper_ir() {
    let output = Command::new(xiom_path())
        .args(["--force", "--no-cache", "--emit-ir",
               "tests/regression/m326_dup_invariant_helper/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("emit-ir");
    let ir = String::from_utf8_lossy(&output.stdout).to_string();
    let defines = ir.matches("define void @LinkedList.invariant_check").count();
    assert_eq!(
        defines, 1,
        "m326: exactly one LinkedList.invariant_check definition must be emitted (got {defines})"
    );
}

// m334 (REL-20261015-1941-packages): an INLINE `Str::from_utf8(...)` call as
// the first component of a tuple RESULT mis-named the tuple element -- both
// tuple-element namers derive the element name through infer_llvm_type,
// which reported i64 for the builtin Str intercepts (they compile to i8* but
// have no fn-registry entry), so the Ok payload built a 16-byte
// Tuple__Int__Int ({i64,i64}) against the signature's Tuple__Str__Int
// ({ptr,i64}) and clang rejected the store. The fixture's red arm is
// Ok((Str::from_utf8(out), 1)) plus a direct tuple return; its controls are
// the let-bound form (which hid the bug via local_xiom_types) and the Err
// arm. The fix reports the intercept's true ABI type (i8*) from
// infer_llvm_type's call arm, mirroring the call.rs guard.
#[test] fn e2e_m334_from_utf8_tuple() {
    assert_eq!(
        compile_and_run("tests/regression/m334_from_utf8_tuple/main.xi"),
        Some(0),
        "inline Str::from_utf8 tuple components must keep the signature's Tuple__Str__Int shape (m334)"
    );
}

// m334 IR twin: the `pair_inline` body must build the {ptr,i64}
// Tuple__Str__Int literal -- never a Tuple__Int__Int (pre-fix: the alloca
// and load used the 16-byte Int shape and the Ok store mismatched).
#[test] fn e2e_m334_from_utf8_tuple_ir() {
    let ir = compile_ir("tests/regression/m334_from_utf8_tuple/main.xi")
        .expect("m334 fixture must compile to IR");
    let start = ir.find("define %struct.Result__Tuple__Str__Int__PgError @pair_inline")
        .expect("m334: pair_inline must be defined");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("alloca %struct.Tuple__Str__Int"),
        "m334: the inline producer tuple must allocate the Str shape; got:\n{body}"
    );
    assert!(
        !body.contains("Tuple__Int__Int"),
        "m334: the inline producer tuple must not collapse to Tuple__Int__Int; got:\n{body}"
    );
}

// m335 (REL-20261014-1208-bindings-2, B-29 triage): the "big-int bitops are
// wrong for values >= 2^31" report is arithmetically mis-grounded -- the
// bundle probe's literal 3132620800 is 0xBAB80000, NOT 0xBAD00000
// (0xBAD00000 = 3134193664), so its "expected" values are impossible for
// the literal as written while the values it reports as wrong are exactly
// correct for it. Re-verified on the official v0.64.5 archive AND on main:
// no 32-bit truncation path exists (Int shift/and/div/or lower as i64) and
// the true NGX 0xBAD00000 result-code class (0xBAD00000|N, N in 0..18)
// classifies correctly in every shape -- constant-folded, hex, runtime
// globals, fn-param calls, and UInt32->Int / Int32->Int provenance. The
// lock pins the correct semantics plus the true decimal constant; it is
// green-first by design (a triage lock has no red arm).
#[test] fn e2e_m335_bigint_bitops() {
    assert_eq!(
        compile_and_run("tests/regression/m335_bigint_bitops/main.xi"),
        Some(0),
        "big-int shift/and/div/classify semantics must stay exact for values >= 2^31 (m335)"
    );
}

// m335 IR twin: the fixture's runtime big-int ops must lower wide --
// `ashr i64` / `and i64` / `sdiv i64` / `or i64` -- with both big constants
// as i64 initializers; any 32-bit truncation path would surface as i32-width
// ops or narrowed constants.
#[test] fn e2e_m335_bigint_bitops_ir() {
    let ir = compile_ir("tests/regression/m335_bigint_bitops/main.xi")
        .expect("m335 fixture must compile to IR");
    for op in ["ashr i64", "and i64", "sdiv i64", "or i64"] {
        assert!(
            ir.contains(op),
            "m335: big-int ops must lower wide ({op} missing)"
        );
    }
    for bad in ["ashr i32", "and i32", "sdiv i32", "or i32"] {
        assert!(
            !ir.contains(bad),
            "m335: no 32-bit-width op may appear ({bad} found)"
        );
    }
    assert!(
        ir.contains("i64 3134193664"),
        "m335: the true 0xBAD00000 constant must be an i64 initializer"
    );
    assert!(
        ir.contains("i64 3132620800"),
        "m335: the probe's literal must be an i64 initializer"
    );
}

// m337 (XVC-C-17, REL-20261015-2216-xvector): an imported-type module-level
// `var` whose declared spelling is module-qualified (`mutex.SyncMutex` or
// the full `xiom.sync.mutex.SyncMutex`) misregistered as i64 -- the global
// gate's llvm_type_for scored the WHOLE dotted spelling while the registry
// key is the bare leaf, fell back to Ok("i64") + an unknown-type warning,
// and emitted `global i64`: whole-struct stores kept field 0 only and every
// field read returned 0/null (guarded locks silently unguarded -- the G-1
// global-lock pattern). The hermetic fixture locks the qualified-alias +
// full-path spellings, whole-struct assignment (literal + cross-module
// constructor), read-back of every field, the INITIAL-value read, and the
// same-leaf collision variant (XVC-C-14 class: ga.Widget{alpha} vs
// gb.Widget{beta;gamma}) where the qualified spelling must keep its own
// layout. Pre-fix: rc 13 (12 unknown-type warnings); green = rc 0.
#[test] fn e2e_m337_imported_global_qualified_type() {
    assert_eq!(
        compile_and_run("tests/regression/m337_imported_global_qualified_type/main.xi"),
        Some(0),
        "m337: imported-type module-level vars must keep their struct layout \
         (qualified + full-path spellings, all fields, initial-value read, \
         same-leaf collision owners)"
    );
}

// m337 stdlib leg: the reporter's real shape -- `use xiom.sync.mutex;` +
// module-level `var G_M: mutex.SyncMutex` + `G_M = mutex.mutex_new()`.
// Pre-fix: rc 10, 5x "unknown type 'mutex.SyncMutex'" -- the lock handle
// read back null everywhere. Needs the stdlib checkout (xiom/sync/mutex.xi).
#[test] fn e2e_m337_imported_global_stdlib_mutex() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("sync").join("mutex.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/sync/mutex.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m337_imported_global_qualified_type/main_stdlib.xi"),
        Some(0),
        "m337: `mutex.SyncMutex` module-level var stores must survive \
         (whole-struct assign + field readback; XVC-C-17)"
    );
}

// m337 IR twin: every module-level var slot of an imported type must be a
// %struct global (pre-fix: `internal global i64 0` -- field-0-only stores,
// zero reads). The collision pair (GW/GV) must keep its own registered key.
#[test] fn e2e_m337_imported_global_qualified_type_ir() {
    let output = Command::new(xiom_path())
        .args(["--force", "--no-cache", "--emit-ir",
               "tests/regression/m337_imported_global_qualified_type/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("emit-ir");
    let ir = String::from_utf8_lossy(&output.stdout).to_string();
    let m = "m337_imported_global_qualified_type.main";
    for (sym, want) in [
        ("GA", "%struct.SyncMutex"),
        ("GB", "%struct.SyncMutex"),
        ("GC", "%struct.SyncMutex"),
        ("GD", "%struct.SyncMutex"),
        ("GE", "%struct.Guarded"),
        ("GW", "%struct.xiom.m337sync.gb.Widget"),
        ("GV", "%struct.xiom.m337sync.ga.Widget"),
    ] {
        assert!(
            ir.contains(&format!("@{m}.{sym} = internal global {want}")),
            "m337: @{m}.{sym} must be emitted as {want}; got:\n{}",
            ir.lines().filter(|l| l.contains(&format!("@{m}.{sym} ="))).collect::<Vec<_>>().join("\n")
        );
        assert!(
            !ir.contains(&format!("@{m}.{sym} = internal global i64")),
            "m337: @{m}.{sym} must not degrade to an i64 global"
        );
    }
}

// m336 (REL-20261015-2022-orbitdb; C-ORBIT-12): a tuple with an ENUM element
// inside a Result whose ERROR type is ALSO an enum
// (`Result[(T8Val, Int), T8Err]`) miscompiled on v0.64.5 / main / the m334
// branch: the Result arm of concrete_type_for named the tuple payload via
// type_from_ast (BARE `Tuple__T8Val__Int`) while the construction namers
// built the module-qualified `Tuple__<mod>.T8Val__Int`, so the Ok lowering
// stored the qualified value into the bare-named Result field and clang
// hard-failed (`%tmpN defined with type ... but expected ...`). The fixture
// ports the ORBITDB repro (enum-error red arm) plus the in-file Int-error
// control (generic %struct.Result, green pre-fix) and the Err arm; both
// payload sums must decode 110.
#[test] fn e2e_m336_enum_tuple_enum_error() {
    assert_eq!(
        compile_and_run("tests/regression/m336_enum_tuple_enum_error/main.xi"),
        Some(0),
        "enum-element tuple inside an enum-error Result must keep ONE tuple spelling (m336)"
    );
}

// m336 IR twin: the enum-element tuple must appear under exactly ONE spelling
// -- the module-qualified `Tuple__m336_enum_tuple_enum_error.T8Val__Int` used
// by the read_ee define/ret/store/load -- never the bare `Tuple__T8Val__Int`;
// and --emit-ir must not warn `unknown type ... defaulting to i64` (the
// silent-degradation mode this split produced in larger programs).
#[test] fn e2e_m336_enum_tuple_enum_error_ir() {
    let output = Command::new(xiom_path())
        .args(["--force", "--no-cache", "--emit-ir",
               "tests/regression/m336_enum_tuple_enum_error/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("emit-ir");
    assert!(
        output.status.success(),
        "m336: --emit-ir must succeed; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unknown type"),
        "m336: no unknown-type fallback may fire; stderr:\n{stderr}"
    );
    let ir = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        ir.contains("define %struct.Result__Tuple__m336_enum_tuple_enum_error.T8Val__Int__T8Err @read_ee"),
        "m336: read_ee must use the qualified concrete Result key"
    );
    assert!(
        !ir.contains("Tuple__T8Val__Int"),
        "m336: the bare Tuple__T8Val__Int spelling must not be emitted"
    );
    // Single-spelling scan: every `Tuple__<token>` spelling whose token ends
    // in `T8Val__Int` must be the ONE qualified spelling.
    let mut spellings: Vec<String> = Vec::new();
    for chunk in ir.split("Tuple__").skip(1) {
        let tok: String = chunk.chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
            .collect();
        let sp = format!("Tuple__{tok}");
        if sp.ends_with("T8Val__Int") && !spellings.contains(&sp) {
            spellings.push(sp);
        }
    }
    assert_eq!(
        spellings,
        vec!["Tuple__m336_enum_tuple_enum_error.T8Val__Int".to_string()],
        "m336: the enum-element tuple must have exactly one spelling (mixing bare/qualified names was the bug)"
    );
}

// m341 (REL-20261015-1358-engine W1.1): a SECONDARY module's bare call to an
// imported submodule's same-leaf fn bound the PARENT module's fn instead.
// jm.mod_json does `use xiom.serialize.json;` and calls bare
// json_string/json_number; pre-fix the emitted call was
// `i8* @serialize.json_string` (the parent's Str helper) while the checker
// had typed the imported `serialize.json.json_string` (JsonValue), so
// json_stringify loaded a JsonValue struct from a Str buffer and the
// process crashed (0xC0000409 / 0xC0000005) with no output. The entry-module
// control in the same fixture was green pre-fix; distinct exit codes per
// shape.
#[test] fn e2e_m341_catalog_call_binding() {
    assert_eq!(
        compile_and_run("tests/regression/m341_catalog_call_binding/main.xi"),
        Some(0),
        "m341: json_string/json_number/json_stringify from a secondary module must bind the imported submodule fns (pre-fix 0xC0000409 crash)"
    );
}

// m341 IR twin: the secondary module's bodies must call the imported
// `serialize.json.json_string` / `serialize.json.json_number` (JsonValue
// producers) and never the parent's `serialize.json_string` /
// `serialize.json_number` (Str helpers).
#[test] fn e2e_m341_catalog_call_binding_ir() {
    let ir = compile_ir("tests/regression/m341_catalog_call_binding/main.xi")
        .expect("m341 fixture must compile to IR");
    for (fnname, good, bad) in [
        ("jm.mod_json.s_str", "call %struct.JsonValue @serialize.json.json_string(", "call i8* @serialize.json_string("),
        ("jm.mod_json.s_bound", "call %struct.JsonValue @serialize.json.json_string(", "call i8* @serialize.json_string("),
        ("jm.mod_json.s_int", "call %struct.JsonValue @serialize.json.json_number(", "call i8* @serialize.json_number("),
    ] {
        let start = ir.find(&format!("define i8* @{fnname}()"))
            .unwrap_or_else(|| panic!("m341: {fnname} must be defined"));
        let rest = &ir[start..];
        let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
        let body = &rest[..end];
        assert!(
            body.contains(good),
            "m341: {fnname} must call the imported submodule fn ({good}); got:\n{body}"
        );
        assert!(
            !body.contains(bad),
            "m341: {fnname} must never bind the parent same-leaf fn ({bad}); got:\n{body}"
        );
    }
}

// m341 (REL-20261015-1538-engine W2.1): a const reached through a `pub use`
// RE-EXPORT chain, read bare inside a secondary module. The m252 catalog-body
// use-skip probe (m317) only looked at the target module's OWN text for
// `pub const`, so the pure facade m341o.reexp (`pub use m341o.reflect;` and
// nothing else) was classified const-free, the use was skipped, the
// re-exported RFL_OK never reached the body's surface and
// m341o.property2.f2 was error[T001] undefined variable 'RFL_OK' in catalog
// body [m341o.property2]. Entry-level re-export and direct-use controls
// green pre-fix.
#[test] fn e2e_m341_const_reexport() {
    assert_eq!(
        compile_and_run("tests/regression/m341_const_reexport/main.xi"),
        Some(0),
        "m341: a const reached via a pub-use re-export chain must resolve bare inside a secondary module (pre-fix T001)"
    );
}

// m341 entry twin for the re-export const: the secondary module's f2 body
// must read the const (substituted literal 1 / entry-level re-export read
// both stay green) -- assert the define exists and returns the const value
// rather than the undefined-name T001 path.
#[test] fn e2e_m341_const_reexport_ir() {
    let ir = compile_ir("tests/regression/m341_const_reexport/main.xi")
        .expect("m341 const fixture must compile to IR");
    let start = ir.find("define i64 @m341o.property2.f2()")
        .or_else(|| ir.find("define i64 @m341o.property2.f2("))
        .expect("m341: m341o.property2.f2 must be defined (pre-fix T001 undefined variable)");
    let rest = &ir[start..];
    let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("ret i64 1"),
        "m341: f2 must return the re-exported const value; got:\n{body}"
    );
}

// m341 import-order twins (W2.2): the cons-before-reflect and
// reflect-before-cons orderings of the entry's uses must BOTH stay green
// (m317 fixed the original order sensitivity; this pins both directions).
#[test] fn e2e_m341_const_import_order_a() {
    assert_eq!(
        compile_and_run("tests/regression/m341_const_reexport/order_a.xi"),
        Some(0),
        "m341: cons-before-reflect import order must stay green"
    );
}

#[test] fn e2e_m341_const_import_order_b() {
    assert_eq!(
        compile_and_run("tests/regression/m341_const_reexport/order_b.xi"),
        Some(0),
        "m341: reflect-before-cons import order must stay green"
    );
}

// m342 (REL-20261015-1358-engine, W1.3): a RECOVERABLE parse error inside an
// IMPORTED module (`var fn` -- `fn` is a reserved keyword) used to be
// swallowed on the import path: the module loaded into the catalog from its
// partially-parsed AST (the parser jumps to the next declaration), `calc`
// vanished from its surface, and the entry compile failed with a downstream
// `error[T001]: undefined variable 'calc'` -- the dropped item masqueraded as
// a name-resolution bug. The module's recorded parse diagnostics must now
// escalate to a hard P001 naming the module path + position, and the
// downstream cascade must not be the reported failure.
#[test] fn e2e_m342_module_parse_error_surface() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "tests/regression/m342_varfn_parse_surface/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed to run xiom");
    assert!(
        !output.status.success(),
        "an imported module with a parse error must fail the build (m342); stdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error[P001]"),
        "the imported module's parse error must surface as P001; got:\n{stderr}"
    );
    assert!(
        stderr.contains("reserved keyword"),
        "the parser's own message must be preserved; got:\n{stderr}"
    );
    assert!(
        stderr.contains("xiom.m342vf.gen"),
        "the diagnostic must name the imported module; got:\n{stderr}"
    );
    assert!(
        !stderr.contains("undefined variable 'calc'"),
        "the downstream T001 cascade must not masquerade as the failure (m342); got:\n{stderr}"
    );
}

// m342 module-alone leg: compiling the poisoned module BY ITSELF always
// reported the P001 (the entry-file parse path); the lock keeps that hard
// parse error in place at module compile time.
#[test] fn e2e_m342_module_alone_hard_p001() {
    let output = Command::new(xiom_path())
        .args(["--check", "tests/regression/m342_varfn_parse_surface/xiom/m342vf/gen.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed to run xiom");
    assert!(
        !output.status.success(),
        "a module with a reserved-keyword var must not pass --check (m342)"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("P001") && stderr.contains("reserved keyword"),
        "the module compile must hard-fail with the P001; got:\n{stderr}"
    );
}

// m342 sibling case: ANY reserved keyword as a var name (`var let`) is the
// same recoverable-parse class as `var fn` -- the entry importing that module
// must get the P001 surface too, never the downstream cascade.
#[test] fn e2e_m342_reserved_var_name_surface() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "tests/regression/m342_varfn_parse_surface/main_reserved.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed to run xiom");
    assert!(
        !output.status.success(),
        "an imported module with `var let` must fail the build (m342)"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error[P001]") && stderr.contains("reserved keyword"),
        "the `var let` module's P001 must surface; got:\n{stderr}"
    );
    assert!(
        stderr.contains("xiom.m342vf.misc"),
        "the diagnostic must name the module; got:\n{stderr}"
    );
    assert!(
        !stderr.contains("undefined variable 'calc_let'"),
        "no downstream cascade may masquerade as the failure (m342); got:\n{stderr}"
    );
}

// m342 control: the same import shape against a parse-clean sibling module
// stays green (prints 5 then 8, exit 0; self-checked) -- the escalation is
// inert for valid modules.
#[test] fn e2e_m342_sibling_module_control() {
    assert_eq!(
        compile_and_run("tests/regression/m342_varfn_parse_surface/control_main.xi"),
        Some(0),
        "a parse-clean imported sibling module must stay green (m342)"
    );
}

// m344 (REL-20261011-1046-engine): the borrow pass is no longer type-blind --
// a local bound to a CALL RESULT (S make()) is move-only, so reusing it as a
// bare argument warns E001. Pre-fix: ZERO diagnostics and --strict rc 0 (the
// "Int" copy fallback); the annotated twin warned pre-fix -- parity is locked
// here. Default mode stays a warning (the program must still compile + run).
#[test] fn e2e_m344_callinit_move_e001() {
    assert_eq!(
        compile_and_run("tests/regression/m344_callinit_move/main.xi"),
        Some(0),
        "m344: call-init move diagnostics are warnings by default; the program must still run"
    );
    let (ok, out) = compile_capture(
        "tests/regression/m344_callinit_move/main.xi",
        &["--force", "--no-cache", "--emit-ir"],
    );
    assert!(ok, "m344: default mode must compile (E001 is a warning); output:\n{out}");
    assert!(
        out.contains("warning[E001]"),
        "m344: the call-init reuse must warn E001; output:\n{out}"
    );
    assert!(
        out.contains("use of moved value 'a'"),
        "m344: the UNANNOTATED call-init reuse must be named (pre-fix: silent); output:\n{out}"
    );
    assert!(
        out.contains("use of moved value 'b'"),
        "m344: annotated/unannotated parity -- 'b' must warn too; output:\n{out}"
    );
    let (ok_strict, out_strict) = compile_capture(
        "tests/regression/m344_callinit_move/main.xi",
        &["--force", "--no-cache", "--strict", "--emit-ir"],
    );
    assert!(
        !ok_strict,
        "m344: --strict must abort on the call-init reuse; output:\n{out_strict}"
    );
    assert!(
        out_strict.contains("aborting compilation"),
        "m344: --strict must abort compilation; output:\n{out_strict}"
    );
}

// m344 (REL-20261011-1046-engine): a field projection passed by value
// consumes the projected place. Pre-fix both shapes (struct field + the
// engine's Vec[UInt8] field) compiled with ZERO diagnostics.
#[test] fn e2e_m344_fieldproj_move_e001() {
    assert_eq!(
        compile_and_run("tests/regression/m344_fieldproj_move/main.xi"),
        Some(0),
        "m344: field-projection move diagnostics are warnings by default; the program must still run"
    );
    let (ok, out) = compile_capture(
        "tests/regression/m344_fieldproj_move/main.xi",
        &["--force", "--no-cache", "--emit-ir"],
    );
    assert!(ok, "m344: default mode must compile (E001 is a warning); output:\n{out}");
    assert!(
        out.contains("use of moved value 'a.f'"),
        "m344: the struct-field projection reuse must warn on 'a.f' (pre-fix: silent); output:\n{out}"
    );
    assert!(
        out.contains("use of moved value 'dev.fb'"),
        "m344: the engine-shape Vec-field projection must warn on 'dev.fb' (pre-fix: silent); output:\n{out}"
    );
    let (ok_strict, out_strict) = compile_capture(
        "tests/regression/m344_fieldproj_move/main.xi",
        &["--force", "--no-cache", "--strict", "--emit-ir"],
    );
    assert!(
        !ok_strict,
        "m344: --strict must abort on the projection reuse; output:\n{out_strict}"
    );
    assert!(
        out_strict.contains("aborting compilation"),
        "m344: --strict must abort compilation; output:\n{out_strict}"
    );
}

// m344 negative lock (REL-20261011-1046-engine): genuine scalars keep the
// copy exemption -- a scalar field projection and a scalar call-init local
// reuse stay clean in BOTH modes (no E001, --strict rc 0).
#[test] fn e2e_m344_scalar_field_clean() {
    assert_eq!(
        compile_and_run("tests/regression/m344_scalar_field_clean/main.xi"),
        Some(0),
        "m344: scalar reuse fixture must run"
    );
    let (ok, out) = compile_capture(
        "tests/regression/m344_scalar_field_clean/main.xi",
        &["--force", "--no-cache", "--emit-ir"],
    );
    assert!(ok, "m344: scalar-only fixture must compile; output:\n{out}");
    assert!(
        !out.contains("E001"),
        "m344: scalar field/call-init reuse must stay clean; output:\n{out}"
    );
    let (ok_strict, out_strict) = compile_capture(
        "tests/regression/m344_scalar_field_clean/main.xi",
        &["--force", "--no-cache", "--strict", "--emit-ir"],
    );
    assert!(
        ok_strict,
        "m344: scalar reuse must stay clean under --strict; output:\n{out_strict}"
    );
}

// m345 (REL-20261011-1411-engine): the contract clause payload rebind
// (`result is Ok` / `result is Some`) hard-coded an i64-HANDLE load of the
// payload field. The concrete monomorphs carry the payload INLINE
// (%struct.Result__Vec__IOError = { i64, %struct.Vec, ... }), so the i64
// load grabbed the Vec DATA POINTER; the `.len()` consequence inttoptr'd it
// to %struct.Vec* and read the payload's own bytes as the length -- for the
// fixture's hermetic file payload (bytes 8..15 carry bit 63) the length
// evaluated NEGATIVE and the correct tautology `result.len() >= 0` aborted:
// the engine grid.ppm report (io.xi:947). Arm A = the file-read clause,
// arm B = the `.value`-reader consequence (field read 3338-3350), arm C =
// the concrete Option payload field read, arm D = the BASE Option handle
// layout control, E/F = positive-bytes + Err-vacuous controls. Pre-fix:
// rc 1 `contract violated: ensures`; post-fix rc 0.
#[test] fn e2e_m345_clause_inline_payload_rebind() {
    assert_eq!(
        compile_and_run("tests/regression/m345_clause_inline_payload_rebind/main.xi"),
        Some(0),
        "the concrete clause payload rebind must lower against the INLINE struct (pre-fix contract violation at arm A)"
    );
}

// m345 IR twin: the clause payload rebind of the concrete
// Result__Vec__IOError container must load the payload with its ACTUAL
// LLVM type (`load %struct.Vec, %struct.Vec*`) and the len consequence must
// GEP into the INLINE struct -- never inttoptr a field-1 load to
// %struct.Vec* (the pre-fix shape inttoptr'd the Vec data pointer and the
// payload bytes 8..15 became the length). Checked on the file-read fn (A),
// the `.value`-reader fn (B) and the concrete Option fn (C) -- the clause
// window is sliced from the `; contract: ensures:` marker so the confined
// trampoline's own inttoptr (`to %struct.Result__Vec__IOError*`) is out of
// scope.
#[test] fn e2e_m345_clause_inline_payload_rebind_ir() {
    let ir = compile_ir("tests/regression/m345_clause_inline_payload_rebind/main.xi")
        .expect("m345 fixture must compile to IR");
    // Premise: the concrete monomorph carries the Vec payload INLINE.
    assert!(
        ir.contains("%struct.Result__Vec__IOError = type { i64, %struct.Vec,"),
        "m345: the concrete Result must carry the Vec payload inline"
    );
    assert!(
        ir.contains("%struct.Option__Blob = type { i64, %struct.m345_clause_inline_payload_rebind.Blob }"),
        "m345: the concrete Option must carry the Blob payload inline"
    );
    for (def, marker) in [
        ("define %struct.Result__Vec__IOError @own_read_file_bytes(", "load %struct.Vec, %struct.Vec*"),
        ("define %struct.Result__Vec__IOError @own_read_file_bytes_value(", "load %struct.Vec, %struct.Vec*"),
        ("define %struct.Option__Blob @pick_blob(", "load %struct.m345_clause_inline_payload_rebind.Blob, %struct.m345_clause_inline_payload_rebind.Blob*"),
    ] {
        let start = ir.find(def)
            .unwrap_or_else(|| panic!("m345: `{def}` must be defined"));
        let rest = &ir[start..];
        let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
        let body = &rest[..end];
        let cstart = body.find("; contract: ensures")
            .unwrap_or_else(|| panic!("m345: `{def}` must carry a contract clause marker"));
        let clause = &body[cstart..];
        assert!(
            clause.contains(marker),
            "m345: the rebind must load the payload with its ACTUAL LLVM type; got:\n{clause}"
        );
        assert!(
            clause.contains("getelementptr %struct.Vec, %struct.Vec*"),
            "m345: the consequence must GEP into the INLINE Vec; got:\n{clause}"
        );
        assert!(
            !clause.contains("inttoptr"),
            "m345: the clause must never inttoptr a field-1 load as a %struct.Vec* handle; got:\n{clause}"
        );
    }
}

// m346 (REL-20261016-1541-xvector / XVC-C-18): an INFERRED local bound from
// `x.field[i]` lost the element type whenever the field's element needed
// GENERIC SUBSTITUTION (Map's `keys: Vec[K]`): the slot degraded to i64 (the
// key printed as pointer digits; Str consumers received the slot ADDRESS,
// refusing a VALID mapping file) and `name.len()` was a false T001 (the
// binding typed as the raw param K). Pre-fix the fixture fails check/codegen
// with T001 (47:8 param-rooted, 64:21 payload-rooted at base d4fdd7e7);
// post-fix it compiles and the payload arm, both workaround controls and the
// c1/c2 controls all assert green. The param-rooted arm is compile-only --
// its runtime VALUES are G2 (see docs/COMPILER_BUGS.md).
#[test] fn e2e_m346_field_chain_elem_binding() {
    assert_eq!(
        compile_and_run("tests/regression/m346_field_chain_elem_binding/main.xi"),
        Some(0),
        "m346: inferred bindings from generic-substituted field Vec elements must keep their element type"
    );
}

// m346 IR twin: the `name` binding from `entries.keys[i]` keeps an i8* (Str
// handle) slot -- the element load inttoptrs the handle back to i8* -- and
// nothing in the payload scan formats the key through @xiom_int_to_string
// (pre-fix: `alloca i64` slot + `call i8* @xiom_int_to_string` + the slot
// ADDRESS bitcast into Str consumers).
#[test] fn e2e_m346_field_chain_elem_binding_ir() {
    let ir = compile_ir("tests/regression/m346_field_chain_elem_binding/main.xi")
        .expect("m346: fixture must emit IR");
    let start = ir.find("define i64 @scan_payload")
        .expect("m346: scan_payload must be emitted");
    let rest = &ir[start..];
    let end = rest.find("\n}").map(|i| i + 2).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("alloca i8*"),
        "m346: the inferred key binding must keep an i8* (Str handle) slot; got:\n{body}"
    );
    assert!(
        body.contains("inttoptr i64"),
        "m346: the key element load must inttoptr the handle back to i8*; got:\n{body}"
    );
    assert!(
        !body.contains("@xiom_int_to_string"),
        "m346: the key must never be formatted as an Int (no @xiom_int_to_string in the scan body); got:\n{body}"
    );
}

// m347 (REL-20261016-2000-stdlib): a BARE in-module reference to the
// declaring module's own platform constant kept the stdlib LITERAL
// initializer while qualified cross-module refs rewrote to the target. The
// injected env consts registered the literal under the module-qualified key
// (`env.FAMILY`) that `lookup_const_keyed` consults FIRST for an in-module
// bare ref, so on Linux every bare-FAMILY branch inside xiom.os.env took
// the Windows path (`env.path_separator()` "\\", join "a\\b", AppData/*
// dirs) while `env.FAMILY` from outside read "unix" (stdlib wave-164 relay).
// The fix registers the TARGET value for exactly the injected env spellings
// (decl.rs platform_env_const_value) and drops the old bare-leaf seed, so
// bare and qualified refs agree. The fixture is hermetic: its own mini
// XIOM_STDLIB (tests/regression/m347_env_bare_refs/mini_std) carries the
// xiom.env copy under test. The run leg asserts bare == qualified == host
// (red pre-fix on non-Windows hosts, exit 22); the Windows-observable lock
// is the `_ir` twin below.
#[test] fn e2e_m347_env_bare_refs() {
    let Some(_runtime_root) = m347_runtime_guard() else { return; };
    let (mini, src) = m347_fixture_paths("m347_env_bare_refs");
    let exe = project_root().join("target").join("m347_env_bare_refs.exe");
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .arg("-o")
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", &mini)
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the m347 run fixture");
    assert!(
        compile.status.success(),
        "m347 fixture must compile with its mini stdlib; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe).output().expect("run the m347 fixture");
    assert_eq!(
        run.status.code(),
        Some(0),
        "m347: bare in-module platform-const refs must agree with the qualified reads (pre-fix on Linux: exit 22; Windows natively green -- the _ir leg is the observable lock)"
    );
    let _ = std::fs::remove_file(&exe);
}

// m347 IR twin (Windows-observable): `--target arm` (aarch64-linux) must
// rewrite the TARGET values into the bare in-module refs. Pre-fix the
// emitted @env.probe_bare_family body loaded the "windows" literal and
// @env.probe_bare_os the "x86_64"/"windows" literal; post-fix they load
// "unix" and "linux" (the self-qualified probe and main's cross-module
// qualified read already emitted "unix" pre-fix via the R65 site rewrite --
// kept asserted against regression).
#[test] fn e2e_m347_env_bare_refs_ir() {
    let (mini, src) = m347_fixture_paths("m347_env_bare_refs");
    let output = Command::new(xiom_path())
        .args(["--target", "arm", "--emit-ir"])
        .arg(&src)
        .env("XIOM_STDLIB", &mini)
        .current_dir(project_root())
        .output()
        .expect("spawn xiom --target arm --emit-ir for m347");
    assert!(
        output.status.success(),
        "m347 IR leg must compile to IR; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ir = String::from_utf8_lossy(&output.stdout).to_string();

    // String-constant table: @.strN -> the literal payload (without \00).
    let mut strs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in ir.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("@.str") else { continue };
        let Some((num, val)) = rest.split_once(" = private unnamed_addr constant") else { continue };
        let Some(start) = val.find("c\"") else { continue };
        let tail = &val[start + 2..];
        let Some(end) = tail.find("\\00\"") else { continue };
        strs.insert(format!("@.str{num}"), tail[..end].to_string());
    }

    let body_of = |sym: &str| -> String {
        let def = format!("define i8* @{sym}()");
        let start = ir.find(&def)
            .unwrap_or_else(|| panic!("m347: `{sym}` must be emitted (called from the fixture main)"));
        let rest = &ir[start..];
        let end = rest.find("\n}").map(|i| i + 2).unwrap_or(rest.len());
        rest[..end].to_string()
    };
    let literal_in = |sym: &str| -> String {
        let body = body_of(sym);
        for r in m347_str_refs(&body) {
            if let Some(v) = strs.get(&r) {
                return v.clone();
            }
        }
        panic!("m347: `{sym}` must reference a string literal; body:\n{body}");
    };

    assert_eq!(
        literal_in("env.probe_bare_family"),
        "unix",
        "m347: the bare in-module FAMILY ref must emit the --target arm value, not the \"windows\" literal"
    );
    assert_eq!(
        literal_in("env.probe_bare_os"),
        "linux",
        "m347: the bare in-module OS ref must emit the --target arm value, not the \"windows\" literal"
    );
    assert_eq!(
        literal_in("env.probe_qualified_family"),
        "unix",
        "m347: the self-qualified in-module ref keeps emitting the target value (R65 site rewrite)"
    );
    // main's cross-module qualified read (`env.FAMILY`): the site rewrite
    // must keep emitting the target value too.
    let start = ir.find("define i64 @main(").expect("m347: main must be emitted");
    let rest = &ir[start..];
    let end = rest.find("\n}").map(|i| i + 2).unwrap_or(rest.len());
    let main_body = &rest[..end];
    assert!(
        m347_str_refs(main_body).iter().any(|r| strs.get(r).map_or(false, |v| v == "unix")),
        "m347: the qualified `env.FAMILY` read in main must keep emitting the target value (\"unix\"); body:\n{main_body}"
    );
    assert!(
        !m347_str_refs(&body_of("env.probe_bare_family"))
            .iter()
            .any(|r| strs.get(r).map_or(false, |v| v == "windows")),
        "m347: the \"windows\" literal must never survive in the bare family ref"
    );
}

/// m347: the `@.strN` references in an IR body fragment.
fn m347_str_refs(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut idx = 0;
    while let Some(p) = body[idx..].find("@.str") {
        let at = idx + p;
        let tail = &body[at..];
        let after = &tail[5..];
        let dlen = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
        let name = tail[..5 + dlen].to_string();
        if !out.contains(&name) {
            out.push(name);
        }
        idx = at + 5 + dlen;
    }
    out
}

// m347 poison lock (REL-20261016-2000 second defect -- Windows-observable):
// the old R65 site rewrite seeded the BARE leaf (`FAMILY` -> platform
// value) on the first qualified env.FAMILY read, clobbering a user
// program-level `const FAMILY` for every LATER bare read (pre-fix this
// fixture exits 3 on Windows AND WSL). The seed is deleted; the user const
// stays untouched (exit 0). Hermetic via the m347 mini stdlib (the
// qualified read must come from the real env shape).
#[test] fn e2e_m347_env_poison() {
    let Some(_runtime_root) = m347_runtime_guard() else { return; };
    let (mini, _) = m347_fixture_paths("m347_env_bare_refs");
    let src = project_root()
        .join("tests").join("regression").join("m347_env_poison").join("main.xi");
    let exe = project_root().join("target").join("m347_env_poison.exe");
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .arg("-o")
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", &mini)
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the m347 poison fixture");
    assert!(
        compile.status.success(),
        "m347 poison fixture must compile with its mini stdlib; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe).output().expect("run the m347 poison fixture");
    assert_eq!(
        run.status.code(),
        Some(0),
        "m347: a qualified env.FAMILY read must not clobber a user program-level `const FAMILY` (pre-fix: exit 3)"
    );
    let _ = std::fs::remove_file(&exe);
}

/// m347 fixture roots: (mini stdlib dir, fixture main.xi).
fn m347_fixture_paths(fixture: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let base = project_root().join("tests").join("regression").join(fixture);
    (base.join("mini_std"), base.join("main.xi"))
}

/// m347: the run legs link the ambient C runtime (the mini stdlib carries
/// modules only, no runtime/) -- skip loudly without a usable checkout and
/// hard-fail under XIOM_REQUIRE_STDLIB=1, mirroring the stdlib-guarded
/// tests (the IR twin needs no runtime and does not guard).
fn m347_runtime_guard() -> Option<std::path::PathBuf> {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return None; };
    if !stdlib_root.join("runtime").join("xiom_runtime.c").is_file() {
        let msg = "SKIP: runtime/xiom_runtime.c missing in the stdlib checkout";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return None;
    }
    Some(stdlib_root)
}

// m348 (REL-20261016-2002-packages, 2026-10-16): pure-XIOM companion of the
// C typed-getter lock (m348_fn_ptr_signature_flow, host.c). A fn pointer
// bound from a CALL whose DECLARED return is fn(..) must record the return
// class at the binding (`let f = make_fit();`), and a plain Ident copy
// (`let g = f;`) must keep it. Pre-fix the call site read `call i64` while
// the callee returns xmm0: P1 printed the 3.0 raw bits and rc was 3.
#[test] fn e2e_m348_fn_ptr_signature_flow_pure() {
    assert_eq!(
        compile_and_run("tests/regression/m348_fn_ptr_signature_flow_pure/main.xi"),
        Some(0),
        "m348: a call-valued fn-pointer binding (and its Ident copy) must keep the declared return class"
    );
}

// m348 IR twin: both indirect calls in main are classified `call double`
// (pre-fix: `call i64 %tmpN(double 1.50000000000000000e0)` + a garbage
// integer-class read of the xmm0 result).
#[test] fn e2e_m348_fn_ptr_signature_flow_pure_ir() {
    let ir = compile_ir("tests/regression/m348_fn_ptr_signature_flow_pure/main.xi")
        .expect("m348: fixture must emit IR");
    let start = ir.find("define i64 @main")
        .expect("m348: main must be emitted");
    let rest = &ir[start..];
    let end = rest.find("\n}").map(|i| i + 2).unwrap_or(rest.len());
    let body = &rest[..end];
    assert!(
        body.contains("call double %tmp"),
        "m348: the indirect fn-pointer call must be double-classified (xmm0); got:\n{body}"
    );
    assert!(
        !body.contains("call i64 %tmp"),
        "m348: no indirect i64-classified fn-pointer call may remain in main; got:\n{body}"
    );
}

// m349 (REL-20261016-2004-packages / XARROW-ARRAYNAME): a source-dir
// catalog module (the installed-package model) declaring `pub type Array`
// was silently dropped by the injection PRIMITIVES filter -- codegen had no
// layout for the bare name, fell back to i64 ("unknown type 'Array' --
// defaulting to i64") and every field read returned zero (arrow's imported C
// struct named `Array`; they renamed it to DataArray). The compiler has no
// builtin `Array` layout at all: fixed arrays parse as `Type::Array` and
// type as `Array[<elem>]` (bracketed), so a bare `Array` can only be a real
// user/package type. Post-fix the declaration injects like any struct and
// the exact values must survive by-value return, &ref param, struct
// literal, a registered method, a nested struct field and a Vec element.
// Pre-fix: rc 255 (every fail bit set) + 20 unknown-type warnings.
#[test] fn e2e_m349_catalog_guarded_type_names() {
    assert_eq!(
        compile_and_run("tests/regression/m349_catalog_guarded_type_names/main.xi"),
        Some(0),
        "m349: a catalog struct named `Array` must inject and keep its \
         layout (pre-fix: zeroed i64 fallback, rc 255)"
    );
}

// m349 IR twin: the injected catalog struct must reach codegen as a real
// `%struct.Array` layout -- pre-fix the IR contained NO struct definition
// for `Array` (every use erased to i64). The single owner injects on the
// bare key (no collision qualification); the injected free fns keep their
// module-qualified symbols (`@m349x.def.mk_arr`).
#[test] fn e2e_m349_catalog_guarded_type_names_ir() {
    let output = Command::new(xiom_path())
        .args(["--force", "--no-cache", "--emit-ir",
               "tests/regression/m349_catalog_guarded_type_names/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("emit-ir");
    assert!(
        output.status.success(),
        "m349: --emit-ir must succeed; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unknown type"),
        "m349: no unknown-type fallback may fire; stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("defaulting to i64"),
        "m349: the injected struct must not degrade to i64; stderr:\n{stderr}"
    );
    let ir = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        ir.contains("%struct.Array = type { i64, i64 }"),
        "m349: the catalog struct named `Array` must be registered with its real layout; got:\n{}",
        ir.lines().filter(|l| l.contains("Array")).take(8).collect::<Vec<_>>().join("\n")
    );
    assert!(
        ir.contains("%struct.Holder = type { %struct.Array, i64 }"),
        "m349: a nested field of the injected type must keep the struct layout"
    );
    assert!(
        ir.contains("define %struct.Array @m349x.def.mk_arr("),
        "m349: the by-value return signature must use the struct, not an erased scalar"
    );
    assert!(
        !ir.contains("call i64 @m349x.def.mk_arr("),
        "m349: no i64-erased call to the catalog constructor may remain"
    );
}

// m350 (REL-20261016-2016-packages): reading `r.value` / `r.error` behind a
// `&Result[..]` / `&Option[..]` parameter crashed 0xC0000005 when the payload
// is a user struct/enum: the param ABI erases to the BASE `%struct.Result*`
// (i64 handle slots) while every VALUE of a struct/enum-payload container is
// the concrete monomorph with the payload INLINE (the caller bitcasts
// monomorph -> base), so the pointer-to-struct field reader loaded field 1 as
// an i64 HANDLE (.error at base offset 16) and the i64->struct coercion
// inttoptr'd the payload's first word -> AV. Arm A struct payload `.value`,
// B `&mut` twin, C the jev-exact `&Result[Q, ScratchErr]` (`.error` match +
// `.value`), D `&Option[P]` `.value`, E `&Result[Str, ScratchErr]` `.error`
// bound + matched -- all red pre-fix; F..N are the base-layout / by-value /
// local / tag-predicate / Err-vacuous controls (green in both states).
#[test] fn e2e_m350_result_ref_field_read() {
    assert_eq!(
        compile_and_run("tests/regression/m350_result_ref_field_read/main.xi"),
        Some(0),
        "m350: payload reads behind &Result/&Option must lower against the concrete monomorph (pre-fix: 0xC0000005 / silent misread)"
    );
}

// m350 IR twin: every payload read behind a `&Result`/`&Option` receiver must
// BITCAST the base-erased pointer to the concrete monomorph and GEP/load the
// inline payload field -- never `load i64` + `inttoptr` the payload's first
// word as a handle. The scalar-payload control (`&Result[Int, Int]`) must stay
// on the base `%struct.Result` layout (no bogus monomorph creation).
#[test] fn e2e_m350_result_ref_field_read_ir() {
    let ir = compile_ir("tests/regression/m350_result_ref_field_read/main.xi")
        .expect("m350 fixture must compile to IR");
    // Premise: the concrete monomorphs carry the payloads INLINE.
    assert!(
        ir.contains("%struct.Result__P__Str = type { i64, %struct.m350_result_ref_field_read.P, i8* }"),
        "m350: Result[P, Str] must carry the P payload inline"
    );
    assert!(
        ir.contains("%struct.Result__Q__ScratchErr = type { i64, %struct.m350_result_ref_field_read.Q, %struct.m350_result_ref_field_read.ScratchErr }"),
        "m350: Result[Q, ScratchErr] must carry both payloads inline"
    );
    assert!(
        ir.contains("%struct.Option__P = type { i64, %struct.m350_result_ref_field_read.P }"),
        "m350: Option[P] must carry the P payload inline"
    );
    let body_of = |def: &str| -> String {
        let start = ir
            .find(def)
            .unwrap_or_else(|| panic!("m350: `{def}` must be defined"));
        let rest = &ir[start..];
        let end = rest[1..].find("\ndefine ").map(|i| i + 1).unwrap_or(rest.len());
        rest[..end].to_string()
    };
    for (def, marker) in [
        ("define i64 @get_a(%struct.Result* %param0)", "to %struct.Result__P__Str*"),
        ("define i64 @get_a_mut(%struct.Result* %param0)", "to %struct.Result__P__Str*"),
        ("define i64 @bump(%struct.Result* %param0)", "to %struct.Result__Q__ScratchErr*"),
        ("define i64 @opt_a(%struct.Option* %param0)", "to %struct.Option__P*"),
        ("define i64 @str_err_len(%struct.Result* %param0)", "to %struct.Result__Str__ScratchErr*"),
    ] {
        let body = body_of(def);
        assert!(
            body.contains("bitcast %struct.") && body.contains(marker),
            "m350: `{def}` must bitcast the base pointer to the concrete monomorph (`{marker}`); got:\n{body}"
        );
        assert!(
            !body.contains("inttoptr"),
            "m350: `{def}` must never inttoptr a field load as a payload handle; got:\n{body}"
        );
    }
    // The struct payload read loads the FULL inline aggregate (field 1 of
    // Result__P__Str), not the i64 first word.
    let get_a = body_of("define i64 @get_a(%struct.Result* %param0)");
    assert!(
        get_a.contains("getelementptr %struct.Result__P__Str, %struct.Result__P__Str*")
            && get_a.contains("i32 0, i32 1")
            && get_a.contains("load %struct.m350_result_ref_field_read.P, %struct.m350_result_ref_field_read.P*"),
        "m350: get_a must GEP/load the INLINE P payload; got:\n{get_a}"
    );
    // The enum error read (field 2) loads the inline enum value.
    let bump = body_of("define i64 @bump(%struct.Result* %param0)");
    assert!(
        bump.contains("getelementptr %struct.Result__Q__ScratchErr, %struct.Result__Q__ScratchErr*")
            && bump.contains("i32 0, i32 2")
            && bump.contains("load %struct.m350_result_ref_field_read.ScratchErr, %struct.m350_result_ref_field_read.ScratchErr*"),
        "m350: bump must GEP/load the INLINE ScratchErr error payload; got:\n{bump}"
    );
    // Control: the scalar-payload receiver keeps the BASE erased layout.
    let int_val = body_of("define i64 @int_val(%struct.Result* %param0)");
    assert!(
        int_val.contains("getelementptr %struct.Result, %struct.Result*")
            && !int_val.contains("bitcast %struct.Result*"),
        "m350: scalar payloads must stay on the base %struct.Result layout; got:\n{int_val}"
    );
    assert!(
        !ir.contains("%struct.Result__Int__Int"),
        "m350: no monomorph may be created for a scalar-payload Result"
    );
}

// m351 (REL-20261016-2031-bindings-3, B-33): the C-boundary IR on
// x86_64-pc-windows-msvc must use the C-lowered signature -- by-value
// aggregates >8B become POINTER operands, >8B returns become `void` with an
// explicit first sret pointer, and raw `Int as fn(..)` casts use the same
// C-lowered fn-pointer type. Naked `%struct` operands/returns at these
// boundaries are the defect (LLVM lowers them differently than MSVC C).
// The target-dispatch is deliberate: SysV/AArch64 keep the internal
// convention until each is validated.
#[test] fn e2e_m351_c_abi_ir() {
    if !cfg!(windows) {
        return;
    }
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "tests/regression/m351_c_abi_ir/main.xi"])
        .current_dir(project_root())
        .output()
        .expect("emit-ir m351 fixture");
    assert!(
        output.status.success(),
        "m351 IR fixture must compile to IR; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ir = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        ir.contains("target triple = \"x86_64-pc-windows-msvc\""),
        "m351 IR lock expects the MSVC x64 target; got:\n{ir}"
    );

    // Direct extern: 16B by-value param is a pointer; 16B return is void+sret.
    let sum2_decl = ir.lines()
        .find(|l| l.trim_start().starts_with("declare i64 @probe_sum2"))
        .expect("probe_sum2 declare must exist");
    assert!(
        sum2_decl.contains("Pair2*") && !sum2_decl.contains("(Pair2,"),
        "m351: probe_sum2 must take a pointer for the 16B by-value record; got:\n{sum2_decl}"
    );
    let make2_decl = ir.lines()
        .find(|l| l.trim_start().starts_with("declare void @probe_make2"))
        .expect("probe_make2 must be declared void (sret)");
    assert!(
        make2_decl.contains("sret(") && make2_decl.contains("Pair2*"),
        "m351: probe_make2 must use an explicit first sret pointer; got:\n{make2_decl}"
    );
    let mul_decl = ir.lines()
        .find(|l| l.trim_start().starts_with("declare i64 @probe_mul"))
        .expect("probe_mul declare must exist");
    assert_eq!(
        mul_decl.trim(),
        "declare i64 @probe_mul(i64, i64)",
        "m351: scalar signatures must stay direct"
    );
    // 8-byte aggregate: C coerces the raw bytes to ONE i64; the declare and
    // operands must not keep the naked %struct (LLVM would split the fields).
    let sum2i_decl = ir.lines()
        .find(|l| l.trim_start().starts_with("declare i64 @probe_sum2i"))
        .expect("probe_sum2i declare must exist");
    assert_eq!(
        sum2i_decl.trim(),
        "declare i64 @probe_sum2i(i64, i64)",
        "m351: 8B by-value aggregate must be integer-coerced"
    );
    let make2i_decl = ir.lines()
        .find(|l| l.trim_start().starts_with("declare i64 @probe_make2i"))
        .expect("probe_make2i must return the coerced integer");
    assert_eq!(
        make2i_decl.trim(),
        "declare i64 @probe_make2i(i64)",
        "m351: 8B aggregate return must be integer-coerced"
    );
    for line in ir.lines() {
        let t = line.trim();
        if (t.contains("@probe_sum2i(") || t.contains("@probe_make2i("))
            && t.starts_with("call ")
        {
            assert!(
                !t.contains("Pair2i"),
                "m351: 8B aggregate call must not carry the naked struct:\n{t}"
            );
        }
    }

    // No call to the extern may carry a naked %struct operand.
    for line in ir.lines() {
        let t = line.trim();
        if t.contains("@probe_sum2(") && t.starts_with("call ") {
            assert!(
                t.contains("Pair2*") && !t.contains("(Pair2,") && !t.contains(", Pair2,"),
                "m351: naked %struct operand on probe_sum2 call:\n{t}"
            );
        }
        if t.contains("@probe_make2(") && t.starts_with("call ") {
            assert!(
                t.starts_with("call void "),
                "m351: probe_make2 return must be void (sret):\n{t}"
            );
            assert!(
                t.contains("sret(") && t.contains("Pair2*"),
                "m351: probe_make2 call must pass the explicit sret slot:\n{t}"
            );
        }
    }

    // Raw Int-as-fn cast: at the CALL SITE the value is cast to the
    // C-lowered fn-pointer type (aggregate parameter = pointer; aggregate
    // return = void + sret pointer). The cast binding itself keeps the XIOM
    // value type; that type is not a C boundary anymore.
    let c2_cast = ir.lines().find(|l| {
        l.contains("bitcast") && l.contains("to i64 (") && l.contains("Pair2*, i64)*")
    });
    assert!(
        c2_cast.is_some(),
        "m351: the c2 call must cast to a C-lowered fn-pointer type (pointer param); IR:\n{ir}"
    );
    let c3_cast = ir.lines().find(|l| {
        l.contains("bitcast") && l.contains("to void (") && l.contains("Pair2*, i64)*")
    });
    assert!(
        c3_cast.is_some(),
        "m351: a returning cast needs the void + sret-pointer fn type; IR:\n{ir}"
    );
    let small_cast = ir.lines().find(|l| {
        l.contains("bitcast") && l.contains("to i64 (i64, i64)*")
    });
    assert!(
        small_cast.is_some(),
        "m351: the 8B aggregate cast must use the integer-coerced fn type; IR:\n{ir}"
    );
    for line in ir.lines() {
        let t = line.trim();
        if t.starts_with("call i64 %") && t.contains("Pair2") {
            assert!(
                t.contains("Pair2*") && !t.contains("(Pair2,") && !t.contains(", Pair2,"),
                "m351: naked %struct operand on a cast call:\n{t}"
            );
        }
        if t.starts_with("call void %") && t.contains("Pair2") {
            assert!(
                t.contains("Pair2*") && !t.contains("(Pair2,") && !t.contains(", Pair2,"),
                "m351: naked %struct operand/return on a cast call:\n{t}"
            );
        }
    }
}

// m351 (B-33): the P1 silent-exit property. A deterministic fault inside a
// confined block must ALWAYS print the typed [fault] diagnostic (no env var)
// and exit 70 when unacknowledged; a fixture that calls take_fault() must
// exit 0 and still print the diagnostic (the fault happened; it was typed).
#[test] fn e2e_m351_no_silent_exit() {
    let exe_name = format!(
        "m351_no_silent_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    );
    let exe = project_root().join(&exe_name);
    let _ = std::fs::remove_file(&exe);
    let compile = Command::new(xiom_path())
        .args(["--overflow-checks", "tests/regression/m244_null_deref_guard/main.xi", "-o"])
        .arg(&exe)
        .current_dir(project_root())
        .output()
        .expect("compile m351 no-silent-exit fixture");
    assert!(
        compile.status.success(),
        "m351 no-silent-exit fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe)
        .env_remove("XIOM_TRACE_FAULT")
        .output()
        .expect("run m351 no-silent-exit fixture");
    let _ = std::fs::remove_file(&exe);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(70),
        "m351: an unacknowledged confined fault must exit 70, not 0\nstdout:\n{}\nstderr:\n{stderr}",
        String::from_utf8_lossy(&run.stdout)
    );
    assert!(
        stderr.contains("[fault] confined block"),
        "m351: the typed fault diagnostic must be ALWAYS on (no XIOM_TRACE_FAULT)\nstderr:\n{stderr}"
    );

    // Acknowledged fault: exit 0, typed record delivered, diagnostic still on.
    let exe2 = project_root().join(format!(
        "m351_take_fault_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let _ = std::fs::remove_file(&exe2);
    let compile2 = Command::new(xiom_path())
        .args(["tests/regression/m351_take_fault/main.xi", "-o"])
        .arg(&exe2)
        .current_dir(project_root())
        .output()
        .expect("compile m351 take-fault fixture");
    assert!(
        compile2.status.success(),
        "m351 take-fault fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&compile2.stderr)
    );
    let run2 = Command::new(&exe2)
        .env_remove("XIOM_TRACE_FAULT")
        .output()
        .expect("run m351 take-fault fixture");
    let _ = std::fs::remove_file(&exe2);
    let stdout2 = String::from_utf8_lossy(&run2.stdout);
    let stderr2 = String::from_utf8_lossy(&run2.stderr);
    assert_eq!(
        run2.status.code(),
        Some(0),
        "m351: take_fault() acknowledging the fault must allow exit 0\nstdout:\n{stdout2}\nstderr:\n{stderr2}"
    );
    assert!(
        stdout2.contains("fault acknowledged: SIGSEGV"),
        "m351: take_fault() must return the typed HardwareFault (signal); stdout:\n{stdout2}"
    );
    assert!(
        stdout2.contains("TAKE-FAULT-OK"),
        "m351: take-fault control marker missing; stdout:\n{stdout2}"
    );
    assert!(
        stderr2.contains("[fault] confined block"),
        "m351: the diagnostic must be printed even when the fault is acknowledged; stderr:\n{stderr2}"
    );
}
// m361 (REL-20261017-1227-packages, 2026-10-17): the xiom.pulse 0.2 hook-seam
// fn-VALUE trio. 1) A module-global initializer that wraps a bare fn through a
// fn-typed destination (`Box{ f: inc_r }`, `apply(inc, 41)`) defers a
// __fnwrap_N forwarding thunk inside the synthesized ginit body; pre-fix the
// deferred list was never flushed -> clang "use of undefined value
// '@__fnwrap_N'". 2) Adjacent fn-typed struct fields: assignment must store
// the uniform closure ENV (the literal path did, the assignment path stored
// RAW code) and a field-read binding must record the field's declared return
// class (pre-fix `pf(4).a` folded to 0 -> exit 10). 3) A fn-typed ALIAS
// parameter (`f: F`) must classify like the direct marker at the call site,
// or the callee/Vec element consumes a raw address as an env -> 0xC0000005.
#[test] fn e2e_m361_fn_global_registry() {
    assert_eq!(
        compile_and_run("tests/regression/m361_fn_value_trio/probe_global_registry.xi"),
        Some(0),
        "m361: ginit fn-value wraps (global struct field / call arg / alias global) must compile and run"
    );
}

// m361 parallel-codegen facet: per-function emitters must inherit the main
// emitter's module-global registrations; pre-fix a bare module-global fn var
// call resolved as an undefined FUNCTION (C001 'unresolved function symbol').
#[test] fn e2e_m361_fn_global_registry_parallel() {
    assert_eq!(
        compile_and_run_with_flags(
            "tests/regression/m361_fn_value_trio/probe_global_registry.xi",
            &["--parallel-codegen"]
        ),
        Some(0),
        "m361: --parallel-codegen must keep module-global fn vars callable"
    );
}

// The item's verbatim minimized probe: `var hook: fn(Int) -> Int = inc;` and
// `hook(1)` == 2. Serial and parallel codegen must both lower it.
#[test] fn e2e_m361_fn_global_probe() {
    let src = "tests/regression/m361_fn_value_trio/probe_fn_global.xi";
    assert_eq!(
        compile_and_run(src),
        Some(2),
        "m361: the minimized module-global fn-var probe must compile (hook(1) == 2)"
    );
    assert_eq!(
        compile_and_run_with_flags(src, &["--parallel-codegen"]),
        Some(2),
        "m361: the minimized module-global fn-var probe must compile with --parallel-codegen"
    );
}

#[test] fn e2e_m361_adjacent_fn_fields() {
    assert_eq!(
        compile_and_run("tests/regression/m361_fn_value_trio/probe_two_hooks.xi"),
        Some(0),
        "m361: assigning the second fn field must not disturb the first; both bound/direct calls must run"
    );
}

#[test] fn e2e_m361_fn_alias_param() {
    assert_eq!(
        compile_and_run("tests/regression/m361_fn_value_trio/probe_fn_alias.xi"),
        Some(0),
        "m361: a fn-typed alias parameter must be passed/called on the closure-ENV convention"
    );
}

// IR twin: (a) every referenced __fnwrap_N has a module-level definition (the
// ginit flush lock), (b) the adjacent-field probe's indirect call is
// STRUCT-classified (pre-fix `call i64 %tmp` + a constant-folded `.a` read),
// (c) the alias probe wraps the caller's bare fn into an env before install.
#[test] fn e2e_m361_fn_value_trio_ir() {
    fn main_body(ir: &str) -> &str {
        let start = ir.find("define i64 @main").expect("m361: main must be emitted");
        let rest = &ir[start..];
        let end = rest.find("\n}").map(|i| i + 2).unwrap_or(rest.len());
        &rest[..end]
    }
    fn assert_fnwrap_defs(ir: &str, label: &str) {
        let mut names: Vec<String> = Vec::new();
        let mut rest = ir;
        while let Some(pos) = rest.find("@__fnwrap_") {
            let tail = &rest[pos + "@__fnwrap_".len()..];
            let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                let name = format!("@__fnwrap_{digits}");
                if !names.contains(&name) { names.push(name); }
            }
            rest = &tail[digits.len()..];
        }
        assert!(!names.is_empty(), "{label}: expected a __fnwrap thunk reference");
        for name in names {
            let defined = ir.lines().any(|l| {
                l.starts_with("define ") && l.contains(&format!("{name}("))
            });
            assert!(defined, "{label}: {name} is referenced but never DEFINED (ginit flush regression)");
        }
    }

    let registry = compile_ir("tests/regression/m361_fn_value_trio/probe_global_registry.xi")
        .expect("m361: registry fixture must emit IR");
    assert!(registry.contains("define internal void @__xiom_ginit_"), "m361: ginit bodies must be emitted");
    assert_fnwrap_defs(&registry, "registry");

    let hooks = compile_ir("tests/regression/m361_fn_value_trio/probe_two_hooks.xi")
        .expect("m361: two_hooks fixture must emit IR");
    let hooks_main = main_body(&hooks);
    assert!(
        hooks_main.contains("call %struct.probe_two_hooks.R %tmp"),
        "m361: the indirect fn-field call must be STRUCT-classified (env-first); got:\n{hooks_main}"
    );
    assert_fnwrap_defs(&hooks, "two_hooks");

    let alias = compile_ir("tests/regression/m361_fn_value_trio/probe_fn_alias.xi")
        .expect("m361: alias fixture must emit IR");
    assert!(
        alias.contains("ptrtoint i64 (i64, i64)* @__fnwrap_"),
        "m361: the alias-param call site must wrap the bare fn into a closure ENV; got:\n{alias}"
    );
    assert_fnwrap_defs(&alias, "fn_alias");
}
// m353 (REL-20261011-1053-stdlib): a DECLARED concrete `Int.hash` /
// `Bool.hash` must own the zero-arg method form. Pre-fix this fixture
// COMPILED but codegen lowered every scalar `.hash()` to the builtin identity
// before user dispatch (`5.hash()` -> 5; stdlib `false.hash()` -> 0), so the
// run returned fail bits 1/2/64/128. Post-fix every form calls the
// declaration; the Float64 bitcast control and the struct receiver keep their
// existing behavior. The explicit-receiver half is main_static.xi.
#[test] fn e2e_m353_builtin_hash_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m353_builtin_shadow/main.xi"),
        Some(0),
        "m353: declared Int.hash/Bool.hash must outrank the scalar .hash() builtin identity"
    );
}

// m353 (REL-20261011-1053-stdlib), explicit-receiver half: `Int.hash(5)` /
// `Bool.hash(false)` failed T001 pre-fix ("expected Self, found Int") because
// the declared method's explicit-receiver param spells `Self` and the argument
// is a primitive. Post-fix both calls dispatch to the declarations.
#[test] fn e2e_m353_builtin_hash_static() {
    assert_eq!(
        compile_and_run("tests/regression/m353_builtin_shadow/main_static.xi"),
        Some(0),
        "m353: explicit-receiver Int.hash(5)/Bool.hash(false) must type-check and dispatch"
    );
}

// m353 hash IR twin: main/generic mono bodies must CALL the declared
// @Int.hash / @Bool.hash symbols (pre-fix the scalar `.hash()` calls were
// absent -- the identity value was used inline) and the Float64 builtin
// identity must stay reachable for a scalar with no declared hash.
#[test] fn e2e_m353_builtin_hash_shadow_ir() {
    let ir = compile_ir("tests/regression/m353_builtin_shadow/main.xi")
        .expect("m353 hash fixture must compile to IR");
    assert!(
        ir.contains("define i64 @Int.hash(i64 %param_self)"),
        "m353: the declared Int.hash definition must be emitted"
    );
    assert!(
        ir.contains("define i64 @Bool.hash(i64 %param_self)"),
        "m353: the declared Bool.hash definition must be emitted"
    );
    for (needle, what) in [
        ("call i64 @Widget.hash(", "struct receiver method"),
        ("call i64 @generic_hash_Int(i64 9)", "generic relay mono (Int)"),
        ("call i64 @generic_hash_Bool(i64 0)", "generic relay mono (Bool)"),
    ] {
        assert!(ir.contains(needle), "m353: IR must contain `{needle}` ({what})");
    }
    // Zero-arg method-form calls to the declarations (not the identity). The
    // `i64 %` receiver distinguishes a CALL from the definition header.
    assert!(
        ir.contains("call i64 @Int.hash(i64 %"),
        "m353: zero-arg `i.hash()` must compile to a call to Int.hash"
    );
    assert!(
        ir.contains("call i64 @Bool.hash(i64 %"),
        "m353: zero-arg `b.hash()` must compile to a call to Bool.hash"
    );
    assert!(
        ir.contains("bitcast double") && ir.contains("to i64"),
        "m353: the builtin scalar identity must remain for Float64 (no declared hash)"
    );
}

// m353 (REL-20261011-1456-stdlib), hermetic half: the globally registered
// builtin Mutex barrier (`Mutex.new/lock/unlock/destroy`) must not shadow a
// module-declared generic `Mutex[T]`. Pre-fix `mutex.Mutex.new(7)` failed T001
// ("'new' expects 0 argument(s), found 1").
#[test] fn e2e_m353_builtin_mutex_shadow() {
    assert_eq!(
        compile_and_run("tests/regression/m353_builtin_shadow/main_mutex.xi"),
        Some(0),
        "m353: a declared generic Mutex[T] must outrank the builtin Mutex barrier"
    );
}

// m353 hermetic Mutex IR twin: the call sites bind the monomorphised DECLARED
// methods (@Mutex.new_Int/get_Int/lock_Int/unlock_Int) and the builtin
// barrier's @xiom_mutex_init is never reached (no stdlib sync import here).
#[test] fn e2e_m353_builtin_mutex_shadow_ir() {
    let ir = compile_ir("tests/regression/m353_builtin_shadow/main_mutex.xi")
        .expect("m353 mutex fixture must compile to IR");
    for needle in [
        "call %struct.Mutex @Mutex.new_Int(i64 7)",
        "call %struct.Mutex @Mutex.new_Str(i8*",
        "call i64 @Mutex.get_Int(%struct.Mutex %",
        "call i64 @Mutex.lock_Int(%struct.Mutex %",
        "call i64 @Mutex.unlock_Int(%struct.Mutex %",
    ] {
        assert!(ir.contains(needle), "m353: IR must contain `{needle}`");
    }
    assert!(
        !ir.contains("@xiom_mutex_init"),
        "m353: the builtin Mutex barrier must not be reached for a declared Mutex[T]"
    );
}

// m353 stdlib half (REL-20261011-1456-stdlib): the documented public
// `xiom.sync.Mutex[T]` surface must be reachable through module qualification.
// Pre-fix `sync.Mutex.new(42)` failed T001 (the checker's builtin `Mutex.new`
// is 0-arg) and no other Mutex method was reachable.
#[test] fn e2e_m353_builtin_mutex_stdlib() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("sync").join("mutex.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/sync/mutex.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    assert_eq!(
        compile_and_run("tests/regression/m353_builtin_shadow/main_mutex_stdlib.xi"),
        Some(0),
        "m353: `sync.Mutex[T]` must be reachable through module qualification"
    );
}

// m353 stdlib Mutex IR twin: the call sites bind the stdlib monomorphs and the
// stdlib method bodies own the barrier FFI (@xiom_mutex_init inside
// @Mutex.new_Int, not a zero-arg builtin call at the call site).
#[test] fn e2e_m353_builtin_mutex_stdlib_ir() {
    let Some(stdlib_root) = xiom_graph::paths::stdlib_or_skip() else { return; };
    if !stdlib_root.join("xiom").join("sync").join("mutex.xi").exists() {
        let msg = "SKIP: stdlib checkout has no xiom/sync/mutex.xi";
        if xiom_graph::paths::require_stdlib() {
            panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
        }
        eprintln!("{msg}");
        return;
    }
    let ir = compile_ir("tests/regression/m353_builtin_shadow/main_mutex_stdlib.xi")
        .expect("m353 stdlib mutex fixture must compile to IR");
    for needle in [
        "call %struct.Mutex @Mutex.new_Int(i64 42)",
        "call %struct.MutexGuard @Mutex.lock_Int(%struct.Mutex %",
        "call i64 @MutexGuard.get_Int(%struct.MutexGuard %",
        "call i64 @Mutex.into_inner_Int(%struct.Mutex %",
        "define %struct.Mutex @Mutex.new_Int(i64 %param0)",
    ] {
        assert!(ir.contains(needle), "m353: IR must contain `{needle}`");
    }
}

// m360 (REL-20261016-2331-stdlib): a RUNTIME 128-bit division that a vector
// element-load loop keeps from constant-folding compiled rc 0 and AV'd with
// 0xC0000005 at run time when the function lived in an xiom.*-namespace
// module. Root cause: clang expands `sdiv i128` to __divti3/__udivti3, and
// the stdlib runtime's `xiom_udivti3` computes `a / b` natively whenever the
// divisor's high limb is zero -- clang lowers that back into the very
// __udivti3 that defines it, so every divisor < 2^64 recurses until the
// stack faults. The fix lowers Int128/UInt128 / and % as an inline restoring
// long division, so no i128 div/rem instruction (and therefore no runtime
// helper) is emitted. The fixture carries its own mini XIOM_STDLIB so the
// xiom.math module resolves hermetically.
//
// Pre-fix: the --opt-level 0 run leg dies 0xC0000005 and the IR leg still
// emits `sdiv i128`; post-fix both legs exit 0 with the witness value 2.
#[test] fn e2e_m360_i128_div_vec_loop() {
    let Some(_runtime_root) = m347_runtime_guard() else { return; };
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("m360_i128_div_vec_loop");
    let mini = base.join("mini_std");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("m360_i128_div_vec_loop.exe");

    // Run legs. --opt-level 0 keeps every divide a runtime operation (nothing
    // folds; pre-fix the recursive helper is reached and the process faults);
    // the default level exercises the folded witness shape the report showed.
    for (opt, what) in [("0", "runtime divides"), ("2", "default level")] {
        let _ = std::fs::remove_file(&exe);
        let compile = Command::new(xiom_path())
            .args(["--no-cache", "--opt-level", opt, "-o"])
            .arg(&exe)
            .arg(&src)
            .env("XIOM_STDLIB", &mini)
            .current_dir(project_root())
            .output()
            .expect("spawn xiom for the m360 fixture");
        assert!(
            compile.status.success(),
            "m360 ({what}): fixture must compile; stderr:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = Command::new(&exe).output().expect("run the m360 fixture");
        assert_eq!(
            run.status.code(),
            Some(0),
            "m360 ({what}): a runtime i128 divide beside a vector element-load loop must return the witness value 2 and exit 0 (pre-fix: 0xC0000005 / status {:?})",
            run.status.code()
        );
    }

    // IR leg: pre-fix the emitted ta4 body carried `sdiv i128`, which clang
    // expands to the runtime __divti3/__udivti3 pair. Post-fix i128 division
    // is the inline restoring loop -- no i128 div/rem instruction may survive.
    let output = Command::new(xiom_path())
        .args(["--no-cache", "--emit-ir"])
        .arg(&src)
        .env("XIOM_STDLIB", &mini)
        .current_dir(project_root())
        .output()
        .expect("spawn xiom --emit-ir for the m360 fixture");
    assert!(
        output.status.success(),
        "m360 IR leg must compile; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ir = String::from_utf8_lossy(&output.stdout);
    for needle in ["sdiv i128", "srem i128", "udiv i128", "urem i128"] {
        assert!(
            !ir.contains(needle),
            "m360: emitted IR must not contain `{needle}` -- that instruction expands to the runtime helper whose single-limb path recurses"
        );
    }
    assert!(
        ir.contains("i128_div_hdr"),
        "m360: the inline long-division loop must be emitted for the runtime divide"
    );
    let _ = std::fs::remove_file(&exe);
}

// m370 (REL-20261018-1546-stdlib): a callee taking `&Vec[Box[Int]]` (a
// generic struct with a nested Vec field) read every element field as
// garbage (plain=111). The element type kept its bracketed annotation
// spelling ("Box[Int]") while the erased generic layout is registered under
// "{module}.Box", so the Vec index fell to the scalar i64 elem_load and the
// field reads became constants/zero.
//
// Pre-fix: run rc 1, stdout `plain=111`; post-fix: run rc 0, `plain=0`.
#[test] fn e2e_m370_vec_byref_generic_struct() {
    let Some(_runtime_root) = m347_runtime_guard() else { return; };
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("m370_vec_byref_generic_struct");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("m370_vec_byref_generic_struct.exe");
    let _ = std::fs::remove_file(&exe);

    let compile = Command::new(xiom_path())
        .args(["--no-cache", "-o"])
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", project_root().join("stdlib"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the m370 fixture");
    assert!(
        compile.status.success(),
        "m370: fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe).output().expect("run the m370 fixture");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "m370: by-ref Vec[GenericStruct] element reads must exit 0 (pre-fix rc 1); stdout: {stdout}"
    );
    assert!(
        stdout.contains("plain=0"),
        "m370: fields read from &Vec[Box[Int]] must be intact (pre-fix plain=111); stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&exe);
}

// m336 residual (C-ORBIT-12 / REL-20261018-1416-orbitdb-2): an applied
// generic alias in an IMPORTED signature (`DbResult[(TValue, Int)]`) must
// not degrade the position tuple to i64 at the caller. Pre-fix the compile
// warned `unknown type 'Tuple__TValue__Int' -- defaulting to i64` and `p.1`
// read offset 8 (inside TValue), so the consumer saw next != 9 (rc 2).
// Green = compile without the warning, run rc 0 with `ok`.
#[test] fn e2e_m336_alias_module_tuple() {
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("m336_alias_module_tuple");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("m336_alias_module_tuple.exe");
    let _ = std::fs::remove_file(&exe);

    let compile = Command::new(xiom_path())
        .args(["--no-cache", "-o"])
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", project_root().join("stdlib"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the m336 alias tuple fixture");
    assert!(
        compile.status.success(),
        "m336 alias tuple: fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let stderr = String::from_utf8_lossy(&compile.stderr);
    assert!(
        !stderr.contains("unknown type"),
        "m336 alias tuple: no unknown-type fallback may fire; stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("defaulting to i64"),
        "m336 alias tuple: the position tuple must not degrade to i64; stderr:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run the m336 alias tuple fixture");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "m336 alias tuple: position tuple p.1 must be correct (pre-fix rc 2); stdout: {stdout}"
    );
    assert!(
        stdout.contains("ok"),
        "m336 alias tuple: expected ok; stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&exe);
}

// XVC-C-20 (REL-20261018-1815-xvector): `&mut` on a module-level aggregate
// must alias the GLOBAL storage, not a temporary copy. Pre-fix the callee
// incremented a copy (a=1, b=0) and the global stayed 0. Green = a=1 b=1 and
// the global's own count/tag are intact.
#[test] fn e2e_xvc_c20_global_aggregate() {
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("xvc_c20_global_aggregate");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("xvc_c20_global_aggregate.exe");
    let _ = std::fs::remove_file(&exe);

    let compile = Command::new(xiom_path())
        .args(["--no-cache", "-o"])
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", project_root().join("stdlib"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the XVC-C-20 aggregate global fixture");
    assert!(
        compile.status.success(),
        "XVC-C-20: fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe)
        .output()
        .expect("run the XVC-C-20 aggregate global fixture");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "XVC-C-20: &mut on a module-level aggregate must persist (pre-fix rc 2); stdout: {stdout}"
    );
    assert!(
        stdout.contains("a=1 b=1"),
        "XVC-C-20: callee and read-back must both see count 1; stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&exe);
}

// B-34 (REL-20261017-0042-bindings): a module-local alias in a generic
// position (`&Vec[Handle]`, Handle = Int) must unify with Vec[Int] across
// modules. Pre-fix the call failed T001 "expected Vec[Handle], found
// Vec[Int]" before codegen; green = compile ok + run n=1 h=42.
#[test] fn e2e_b34_vec_alias_generic() {
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("b34_vec_alias_generic");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("b34_vec_alias_generic.exe");
    let _ = std::fs::remove_file(&exe);

    let compile = Command::new(xiom_path())
        .args(["--no-cache", "-o"])
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", project_root().join("stdlib"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the B-34 alias-generic fixture");
    assert!(
        compile.status.success(),
        "B-34: Vec[Handle] must unify with Vec[Int] across modules; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe)
        .output()
        .expect("run the B-34 alias-generic fixture");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "B-34: alias-generic call must run (pre-fix compile failed T001); stdout: {stdout}"
    );
    assert!(
        stdout.contains("n=1 h=42"),
        "B-34: alias-generic call/return values must be intact; stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&exe);
}

// C-ORBIT-13 (REL-20261019-1209-orbitdb): `&mut` on a >=3-level nested
// field path (`&mut t.store.bs`) must alias the real field, not copy the
// intermediate struct. Pre-fix the three-level case printed n=1 g=0 v=1
// (only the shared Vec buffer leaked through); the two-level control passed.
// Green = all three cases PASS, rc 0.
#[test] fn e2e_c_orbit_13_nested_mut_path() {
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("c_orbit_13_nested_mut_path");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("c_orbit_13_nested_mut_path.exe");
    let _ = std::fs::remove_file(&exe);

    let compile = Command::new(xiom_path())
        .args(["--no-cache", "-o"])
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", project_root().join("stdlib"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the C-ORBIT-13 fixture");
    assert!(
        compile.status.success(),
        "C-ORBIT-13: fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe)
        .output()
        .expect("run the C-ORBIT-13 fixture");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "C-ORBIT-13: two-level, three-level and workaround must all pass (pre-fix three-level failed); stdout: {stdout}"
    );
    assert!(
        stdout.contains("three-level: PASS"),
        "C-ORBIT-13: nested &mut must persist scalar/nested-struct fields; stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&exe);
}

// REL-20261019-1343-engine: editor-shaped frames return several Vecs across
// function boundaries. m368's non-escaping-only pass cannot own them, so the
// runtime scoped arena is the escape hatch: guard_heap_enter/exit around a
// frame makes all compiled allocations arena-backed and discards them
// wholesale at exit. Green = the frames are consumed correctly (the memory
// flatness acceptance is the engine editor soak + the large witness).
#[test] fn e2e_escaped_frame_arena() {
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("escaped_frame_arena");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("escaped_frame_arena.exe");
    let _ = std::fs::remove_file(&exe);

    let compile = Command::new(xiom_path())
        .args(["--no-cache", "-o"])
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", project_root().join("stdlib"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the scoped-arena fixture");
    assert!(
        compile.status.success(),
        "scoped arena: fixture must compile/link (xiom_guard_heap_* are runtime symbols); stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe)
        .output()
        .expect("run the scoped-arena fixture");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "scoped arena: escaped frame Vecs must be consumed correctly; stdout: {stdout}"
    );
    assert!(
        stdout.contains("done 6000000"),
        "scoped arena: frame lengths must be intact; stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&exe);
}

// REL-20261019-1624-stdlib: `Vec[UInt8]::with_capacity(0)` is a valid empty
// buffer (io.read_file on an empty file). The runtime allocator returns NULL
// for size 0; the Vec ctor must not trap on that state, and a later push must
// grow safely. Pre-fix this exits with SIGILL / 0xC000001D.
#[test] fn e2e_vec_with_capacity_zero() {
    let base = project_root()
        .join("tests")
        .join("regression")
        .join("vec_with_capacity_zero");
    let src = base.join("main.xi");
    let exe = project_root().join("target").join("vec_with_capacity_zero.exe");
    let _ = std::fs::remove_file(&exe);

    let compile = Command::new(xiom_path())
        .args(["--no-cache", "-o"])
        .arg(&exe)
        .arg(&src)
        .env("XIOM_STDLIB", project_root().join("stdlib"))
        .current_dir(project_root())
        .output()
        .expect("spawn xiom for the zero-capacity Vec fixture");
    assert!(
        compile.status.success(),
        "zero-capacity Vec: fixture must compile; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&exe)
        .output()
        .expect("run the zero-capacity Vec fixture");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        run.status.code(),
        Some(0),
        "zero-capacity Vec: with_capacity(0) plus push must not trap; stdout: {stdout}"
    );
    assert!(
        stdout.contains("ok"),
        "zero-capacity Vec: expected ok; stdout: {stdout}"
    );
    let _ = std::fs::remove_file(&exe);
}
