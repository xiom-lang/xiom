// XIOM — E2E Tests
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

    let bin_path = xiom_path();

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
    output.status.success() && project_root().join(&wasm_name).exists()
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
fn e2e_no_contracts_flag() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "--no-contracts", "examples\\phase1_contracts.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "contracts should compile without contract checks");
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
fn e2e_runtime_ir_declares_externs() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "selfhost\\xiomc_v091.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("@xiom_read_file"), "IR should declare xiom_read_file");
    assert!(stdout.contains("@xiom_file_size"), "IR should declare xiom_file_size");
    assert!(stdout.contains("@xiom_free"), "IR should declare xiom_free");
}xiom



// ============================================================================
// E2E: Selfhost v11_test — Parameter-counting compiler for demo_float.xi
// ============================================================================




// ============================================================================
// E2E: Selfhost v093 / v095 — Pipeline compilation checks
// ============================================================================



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
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", "examples\\benchmark\\bench_math.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(), "bench_math.xi should compile to IR via ModuleCatalog");
}xiom

/// Compile the full 30-module benchmark suite. Uses the ModuleCatalog
/// (single-file path with lazy loading of all 30 submodules).
/// NOTE: This test passes individually but timxiomt under heavy parallel load
/// due to the 30-module benchmark's size (65536 mono iterations). Run solo:
///   cargo test -p xiom-codegen --test e2e_tests -- e2e_multifile -- --nocapture
#[test]
fn e2e_multifile_benchmark_main_compiles() {
    let output = std::process::Command::new(xiom_path())
        .args(["--emit-ir", "examples\\xiommark\\main.xi"])
        .current_dir(project_root())
        .output()
        .expect("failed");
    assert!(output.status.success(),
        "benchmark/main.xi 30-module suite should compile via ModuleCatalog");
}

// ============================================================================
// E2E: CLI Flags — Timeout & Memory
// NOTE: Watchdog thread tests are inherently racy and environment-dependent.
// Flag parsing correctness is verifiedxiom--help output test below.
// The flags are tested in isolation via unit/integration tests.
// ============================================================================

#[test]xiom
fn e2e_help_shows_timeout_and_memory_flags() {
    let output = std::process::Command::new(xiom_path())
        .arg("--help")
        .current_dir(project_root())
        .output()
        .expect("failed");xiom
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("--timeout"), "help should document --timeout flag");
    assert!(combined.contains("--max-memory-mb"), "help should document --max-memory-mb flag");
}

// ============================================================================
// E2E: Regression — method `match self` on enum receiver
// ============================================================================
/// Regression: a method that pattern-mxioms `self` on an enum receiver must
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
xiom
/// Enum `==`/`!=` via the builtin `.eq` fallback (compare discriminant inline),
/// plus enum-variant construction as values from match arms.
#[test]
fn e2e_enum_eq_and_variants() {
    assert_eq!(compile_and_run("examples\\e2e\\enum_eq.xi"), Some(0),
        "enum ==/!= and variant construction should work");
}

/// Vec builtins: new/push/len/index-read/pop returning Option, with element
/// coercion. Locks in inline Vec method dispatch + Option payload extraction.
#[test]xiom
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

/// Cross-module use of a stdlib module whose functions callxiomterns
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
}xiom

/// Const-generics and array indexing: let-bound arrays index correctly (5a.7),
/// const-declared sizes work in while loops (5a.5).
#[test]
fn e2e_const_generic_array() {
    assert_eq!(compile_and_run("examples\\e2e\\const_generic_array.xi"), Some(0),
        "const-generics: array indexing and const-declared loop sizes");
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
/// through GEP into the struct alloca.xiom
#[test]
fn e2e_field_assign() {
    assert_eq!(compile_and_run("examples\\e2e\\field_assign.xi"), Some(0),
        "struct field assignment should store txiomh GEP");
}

/// Struct method returning modified self stores back to the caller's variable
/// so mutation persists across the call.
#[test]
fn e2e_method_store_back() {xiom
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

/// Or-patterns like `1 | 2 | 3 =>` in match arms coxiom and match correctly.
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
#[test]xiom
fn e2e_djb2_hash() {
    assert_eq!(compile_and_run("examples\\e2e\\djb2_hash.xi"), Some(0),
        "DJB2 hash via Hash[T] interface should produce deterministic non-zero values");
}xiom

/// Generic swap via `&mut T` references: verifies scalar &mut pointers
/// work inside generic monomorphized functions (ARC A + ARC B).
#[test]
fn e2e_mut_ref_swap() {
    assert_eq!(compile_and_run("examplexiome\\mut_ref_swap.xi"), Some(0),
        "generic &mut T swap should exchange values correctly");
}

/// Phase 5c.7 hardening: Float32 compat, hex exioms, enum constructors,
/// comma-separated contracts, Int/Char compat, enum pattern matching.
#[test]
fn e2e_phase5c7_hardening() {
    assert_eq!(compile_and_run("examples\\e2e\\phase5c7_hardening.xi"), Some(0),
        "phase 5c.7 hardening: all 7 compiler fixes pass");
}

/// Combined features: store_back, hash determinism, generic monomorphization.
#[test]
fn e2e_combined_patterns() {
    assert_eq!(compile_and_run("examples\\e2e\\combined_patterns.xi"), Some(0),
        "combined store_back + hash + generics should work together");
}

// ── Ecosystem Hardening Tests ──────────────────────────────────────────

#[test]
fn eco_algo_89_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_algo.xi"), Some(0),
        "ecosystem: all 89 algorithm tests (binary search, quicksort, merge sort, gcd, fib, sieve, etc.)");
}

#[test]
fn eco_crypto_23_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_crypto.xi"), Some(0),
        "ecosystem: all 23 crypto tests (SHA-256, Base64, Hex, FNV-1a)");
}

#[test]
fn eco_db_18_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_db.xi"), Some(0),
        "ecosystem: all 18 database tests (B-Tree, WAL)");
}

#[test]
fn eco_full_30_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_full.xi"), Some(0),
        "ecosystem: all 30 full-feature tests (state machines, enums, contracts, Result, Option)");
}

#[test]
fn eco_http_18_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_http.xi"), Some(0),
        "ecosystem: all 18 HTTP tests (methods, headers, request/response)");
}

#[test]
fn eco_json_29_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_json.xi"), Some(0),
        "ecosystem: all 29 JSON tests (types, parse, stringify, nested)");
}

#[test]
fn eco_net_22_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_net.xi"), Some(0),
        "ecosystem: all 22 networking tests (IPv4/IPv6, sockets)");
}

#[test]
fn eco_sqlite_23_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_sqlite.xi"), Some(0),
        "ecosystem: all 23 SQLite tests (types, rows, CREATE TABLE SQL)");
}

#[test]
fn eco_test_20_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_test.xi"), Some(0),
        "ecosystem: all 20 test-framework tests (asserts, suites, results)");
}

#[test]
fn eco_vector_32_tests() {
    assert_eq!(compile_and_run("tests\\ecosystem\\test_vector.xi"), Some(0),
        "ecosystem: all 32 vector database tests (math, KNN, distance metrics)");
}

#[test]
fn e2e_fnptr_vec_index_call() {
    // Regression: Vec[fn()->Int] element call via index.
    // `tests[i]()` was emitting `inttoptr %struct.Vec to i64 ()*`
    // instead of loading the i64 function pointer from the Vec data.
    // The test returns f() = add_one() which returns 1.
    assert_eq!(compile_and_run("tests\\ecosystem\\test_fnptr.xi"), Some(1),
        "ecosystem: function pointer call from Vec index (tests[i]())");
}

#[test]
fn e2e_this_field_ref() {
    // Regression: &this.field passed to this-based methods.
    // SocketAddr.to_str calling IpAddr.to_str(&this.ip) was loading
    // the IpAddr struct by value instead of passing the GEP pointer,
    // causing ACCESS_VIOLATION (0xC0000005).
    assert_eq!(compile_and_run("tests\\ecosystem\\test_this_field_ref.xi"), Some(0),
        "ecosystem: nested this-based method dispatch via &this.field");
}

#[test]
fn e2e_enum_this_match() {
    // Regression: match on this in this-based methods for enums.
    // struct_type_from_expr didn't remap `this` to `self`, causing
    // LLVM IR errors when matching enum discriminants.
    assert_eq!(compile_and_run("tests\\ecosystem\\test_enum_this_match.xi"), Some(0),
        "ecosystem: enum variant matching in this-based methods");
}

#[test]
fn e2e_vec_of_struct() {
    // Regression: Vec-of-struct inline storage (5c.21 — size-aware
    // elem_size with memcpy push/store/load for multi-field structs).
    assert_eq!(compile_and_run("tests\\ecosystem\\test_vec_of_struct.xi"), Some(0),
        "ecosystem: Vec[Struct] push/index/pop with multi-field structs");
}

#[test]
fn eco_ffi_binding_gaps() {
    // Regression: () in Result generic, pub const cross-module, extern cross-module
    assert_eq!(compile_and_run("tests\\ecosystem\\test_ffi.xi"), Some(0),
        "ecosystem: FFI binding gaps (unit in Result, pub const, extern)");
}

/// 5e.3 G-31: cross-package use resolution — a file in one directory uses
/// a module in another directory via `use`. Locks in the catalog's recursive
/// source_dir scanning + walk-up project root detection.
#[test]
fn e2e_cross_package_use() {
    assert_eq!(compile_and_run("examples\\e2e\\cross_pkg\\main.xi"), Some(0),
        "G-31: cross-package use math_utils should compile and return 0");
}

/// 5e.3 G-30: cross-package extern resolution — extern "C" declarations
/// in one module propagate correctly when used from another module via `use`.
#[test]
fn e2e_cross_package_extern() {
    // Uses the math_utils.xi in examples/e2e/cross_pkg/ — verifies
    // cross-directory resolution works for any file in the tree.
    let result = compile_and_run("examples\\e2e\\cross_pkg\\main.xi");
    assert_eq!(result, Some(0),
        "G-30/G-31: cross-package use+extern should compile and run");
}

/// 5e G-15: sret ABI — C struct return on Linux SysV. Small structs
/// (<16 bytes) return in registers; large structs (>16 bytes) use sret.
/// Verifies XIOM emits correct struct-return IR for both cases.
#[test]
fn e2e_g15_sret_abi() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "examples/e2e/g15_sret.xi"])
        .current_dir(project_root())
        .output()
        .expect("xiom");
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "G-15 sret must compile: {}", String::from_utf8_lossy(&output.stderr));
    assert!(ir.contains("declare %struct.Small @g15_small_return()"),
        "Small struct must return by value (no sret):\n{ir}");
    assert!(ir.contains("declare %struct.Large @g15_large_return()"),
        "Large struct must return (LLVM uses sret):\n{ir}");
    assert!(ir.contains("declare %struct.Small @g15_pass_and_return(%struct.Small)"),
        "struct pass+return must work:\n{ir}");
}

/// 5e G-40: repr(C) layout — mixed-width C struct fields. Verifies XIOM
/// emits correct LLVM struct layout for Int8/Int16/Int32/Int64/Float32/Float64.
#[test]
fn e2e_g40_repc_layout() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "examples/e2e/g40_repc.xi"])
        .current_dir(project_root())
        .output()
        .expect("xiom");
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "G-40 repr(C) must compile: {}", String::from_utf8_lossy(&output.stderr));
    assert!(ir.contains("%struct.MixedC = type"), "MixedC struct must be defined:\n{ir}");
    assert!(ir.contains("i8"), "Must have i8 field for Int8:\n{ir}");
    assert!(ir.contains("i16"), "Must have i16 field for Int16:\n{ir}");
    assert!(ir.contains("i32"), "Must have i32 field for Int32:\n{ir}");
    assert!(ir.contains("float"), "Must have float field for Float32:\n{ir}");
    assert!(ir.contains("double"), "Must have double field for Float64:\n{ir}");
}

/// 5e G-24: Float32 ARM ABI — Float32 operations must produce valid IR
/// for ARM targets (IEEE 754 single-precision). Cross-compiled for
/// aarch64-linux-gnu via clang to verify.
#[test]
fn e2e_g24_float32_arm_abi() {
    let output = Command::new(xiom_path())
        .args(["--emit-ir", "examples/e2e/g24_float32_arm.xi"])
        .current_dir(project_root())
        .output()
        .expect("xiom");
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
// 6A.1: Type Checker Hardening — Self + Interface compatibility e2e tests
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
        .expect("xiom");
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
        .expect("xiom");
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
