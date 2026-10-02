// XIOM -- Selfhost differential gate (Phase 0: T1/T2/T3 tiers + corpus manifest)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The selfhost compiler is built in phases (docs/SELFHOST_PLAN.md). This
// harness is its parity gate over a deterministic corpus:
//
//   T1  feature counts + function names   -- smoke gate during development
//   T2  normalized IR equality            -- strip %tmpN / @.strN numbers
//   T3  exact line-by-line IR equality    -- phase completion gate
//
// Tier selection: XIOM_SELFHOST_DIFF_TIER=1|2|3 (default 1). Tiers STACK:
// T2/T3 also run the T1 checks first. T2/T3 are implemented from Phase 0 but
// stay unreachable until the emitter port advances (the Phase 0 skeleton
// emits a stub module), so the Phase 0 gate is T1 green.
//
// Selfhost side: selfhost/src/main.xi is compiled ONCE per test process with
// the Rust compiler into target/selfhost/ and invoked as
// `xiomc-self <source.xi>` with IR on stdout. This replaces the
// xiomc_v10.xi temp-source-patch runner (no repo-root temp files, no
// embedded input path).
//
// Phase 0 expectations: the skeleton driver emits a well-formed IR module
// (XIOM header + `define`) for every corpus file; per-entry fn-name-match
// expectations start at 0.0 and are RAISED per phase as parity lands. The
// corpus lists are HARDCODED and sorted (no runtime globbing, no
// filesystem-order flakiness).
//
// Gate command: cargo test -p xiom-codegen --test full_diff_tests
//
// Phase 1 (lexer parity) adds `diff_tokens`: the Rust and selfhost
// `--dump-tokens` outputs must match line-for-line over the corpus.
// `crates/xiom/src/main.rs::dump_tokens` owns the format definition;
// `selfhost/src/lexer.xi::dump_tokens` mirrors it (including the Tk-prefixed
// TokenKind variant names mapped back to the Rust tags).
//
// Phase 2 (parser parity) adds `diff_ast`: the Rust and selfhost `--dump-ast`
// outputs must match line-for-line over the corpus.
// `crates/xiom/src/main.rs::dump_ast` (AstDump) owns the format definition;
// `selfhost/src/parser.xi` + `ast_dump.xi` mirror it. Green over the 83-file
// corpus (Phase 2 gate, docs/checklists/selfhost-phase2.md).
//
// Phase 3 (checker parity) adds `diff_check`: the Rust and selfhost
// `--dump-check` outputs (canonical checker diagnostics) must match
// line-for-line over the corpus, AND both must match the committed
// `selfhost/tests/check_negative/*.expected` manifests. Accept cases
// (`CHECK-OK` expected) pin the legacy/permissive behaviors; negative cases
// pin diagnostic text, order and spans. Format ownership:
// `crates/xiom/src/main.rs::dump_check`; the selfhost mirror is
// `selfhost/src/checker.xi::dump_check`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

// ============================================================================
// Corpus manifest
// ============================================================================

#[derive(Clone, Copy)]
struct CorpusEntry {
    /// Repo-root-relative path, forward slashes.
    path: &'static str,
    /// Minimum `define ` count in the Rust compiler's IR (catches a corpus
    /// file silently degrading to an empty module).
    min_rust_fns: usize,
    /// Minimum `define ` count in the selfhost IR. Phase 0: the skeleton
    /// emits a stub module with one function.
    min_self_fns: usize,
    /// Minimum fraction of Rust fn names that must also appear in the
    /// selfhost IR. Phase 0: 0.0 (stub); raised per phase as parity lands.
    min_name_match: f64,
}

const fn ent(path: &'static str) -> CorpusEntry {
    CorpusEntry { path, min_rust_fns: 1, min_self_fns: 1, min_name_match: 0.0 }
}

/// tests/regression/m37_* (49 files, sorted) -- the widest regression sweep.
const M37_FILES: &[&str] = &[
    "tests/regression/m37_bug43_result_f64_payload.xi",
    "tests/regression/m37_bug44_str_deref.xi",
    "tests/regression/m37_bug45_iface_method_generic.xi",
    "tests/regression/m37_bug46_generic_struct_ref.xi",
    "tests/regression/m37_bug47_ref_params_leak.xi",
    "tests/regression/m37_bug48_associated_generic_vec.xi",
    "tests/regression/m37_bug49_fn_param_impl_collision.xi",
    "tests/regression/m37_bug50_ptr_container_name.xi",
    "tests/regression/m37_bug51_option_struct_payload.xi",
    "tests/regression/m37_bug52_map_enum_values.xi",
    "tests/regression/m37_bug53_array_ref_param.xi",
    "tests/regression/m37_bug53_array_ref_write.xi",
    "tests/regression/m37_bug55_payload_loop.xi",
    "tests/regression/m37_bug55_unsafe_ptr_capture.xi",
    "tests/regression/m37_bug56_ensure_expr_body.xi",
    "tests/regression/m37_catalog_boundary.xi",
    "tests/regression/m37_catmod.xi",
    "tests/regression/m37_const_array.xi",
    "tests/regression/m37_contract_pass.xi",
    "tests/regression/m37_debug_intrinsics.xi",
    "tests/regression/m37_else_if.xi",
    "tests/regression/m37_f128.xi",
    "tests/regression/m37_float_precision.xi",
    "tests/regression/m37_from_bytes_fn.xi",
    "tests/regression/m37_global_field_write.xi",
    "tests/regression/m37_global_fn_init.xi",
    "tests/regression/m37_gzip_roundtrip.xi",
    "tests/regression/m37_index_arith.xi",
    "tests/regression/m37_inline_call_concat.xi",
    "tests/regression/m37_labeled_loops.xi",
    "tests/regression/m37_loop_capture.xi",
    "tests/regression/m37_match_float_payload.xi",
    "tests/regression/m37_nan_ieee.xi",
    "tests/regression/m37_nested_vec.xi",
    "tests/regression/m37_numeric_policy.xi",
    "tests/regression/m37_opt_payload_value.xi",
    "tests/regression/m37_payload_ref.xi",
    "tests/regression/m37_ptr_cast.xi",
    "tests/regression/m37_ref_mut.xi",
    "tests/regression/m37_round6_path_gzip.xi",
    "tests/regression/m37_round7_vec_pop_slot.xi",
    "tests/regression/m37_short_circuit.xi",
    "tests/regression/m37_shr_builtin.xi",
    "tests/regression/m37_simd_runtime.xi",
    "tests/regression/m37_str_int_concat.xi",
    "tests/regression/m37_structural_eq.xi",
    "tests/regression/m37_tuple_struct.xi",
    "tests/regression/m37_u128.xi",
    "tests/regression/m37_vec_f64.xi",
];

/// examples/catfix/*.xi (7 files) -- local-module catalog fixtures.
const CATFIX_FILES: &[&str] = &[
    "examples/catfix/b9main.xi",
    "examples/catfix/b9mod.xi",
    "examples/catfix/circ_a.xi",
    "examples/catfix/circ_b.xi",
    "examples/catfix/circ_main.xi",
    "examples/catfix/main.xi",
    "examples/catfix/vecmod.xi",
];

/// examples/phase1_*.xi (16 files) -- the original phase-1 conformance set.
const PHASE1_FILES: &[&str] = &[
    "examples/phase1_async.xi",
    "examples/phase1_async_spawn.xi",
    "examples/phase1_contracts.xi",
    "examples/phase1_derive.xi",
    "examples/phase1_derive_enum.xi",
    "examples/phase1_enum.xi",
    "examples/phase1_error.xi",
    "examples/phase1_full.xi",
    "examples/phase1_generics.xi",
    "examples/phase1_hardening.xi",
    "examples/phase1_impl_trait.xi",
    "examples/phase1_interface.xi",
    "examples/phase1_modules.xi",
    "examples/phase1_ownership.xi",
    "examples/phase1_selfhost.xi",
    "examples/phase1_stress.xi",
];

/// The v10-era stress/benchmark examples, kept for continuity with the old
/// diff suite (benchmark_selfhost + stress_body_parser were Rust-side-only
/// checks there; they are full corpus entries here).
const EXTRA_EXAMPLE_FILES: &[&str] = &[
    "examples/benchmark_selfhost.xi",
    "examples/demo_float.xi",
    "examples/diff_test.xi",
    "examples/stress_body_parser.xi",
    "examples/stress_borrow_10level.xi",
    "examples/stress_derive_50field.xi",
    "examples/stress_float_matrix.xi",
    "examples/stress_generic_5chain.xi",
];

/// Deterministic corpus: hardcoded groups, sorted by path.
fn corpus() -> Vec<CorpusEntry> {
    let mut v = Vec::new();
    for p in ["tests/regression/m33_z14.xi", "tests/regression/m34_d01.xi"] {
        v.push(ent(p));
    }
    for &p in M37_FILES {
        v.push(ent(p));
    }
    for &p in CATFIX_FILES {
        v.push(ent(p));
    }
    for &p in PHASE1_FILES {
        v.push(ent(p));
    }
    for &p in EXTRA_EXAMPLE_FILES {
        v.push(ent(p));
    }
    // The smoke_guard_fault fixture lives in the stdlib checkout (the old
    // checklist path examples/stdlib_smoke/ never existed).
    v.push(ent("stdlib/tests/smoke/smoke_guard_fault.xi"));
    v.sort_by(|a, b| a.path.cmp(b.path));
    v
}

/// Phase 3 checker manifest: committed `.expected` files are the source of
/// truth, asserted against BOTH drivers. Accept cases (`CHECK-OK`) pin
/// legacy/permissive behavior; negative cases pin message text and order.
const NEGATIVE_CASES: &[&str] = &[
    "selfhost/tests/check_negative/arg_count_mismatch.xi",
    "selfhost/tests/check_negative/arg_type_mismatch.xi",
    "selfhost/tests/check_negative/assign_to_let_ok.xi",
    "selfhost/tests/check_negative/bad_if_condition.xi",
    "selfhost/tests/check_negative/contract_clause_non_bool.xi",
    "selfhost/tests/check_negative/div_zero_literal.xi",
    "selfhost/tests/check_negative/let_type_mismatch.xi",
    "selfhost/tests/check_negative/match_qualified_variant_catchall_ok.xi",
    "selfhost/tests/check_negative/match_then_second_fn_scope_ok.xi",
    "selfhost/tests/check_negative/return_type_mismatch.xi",
    "selfhost/tests/check_negative/undefined_fn_call.xi",
    "selfhost/tests/check_negative/undef_var.xi",
    "selfhost/tests/check_negative/unknown_field.xi",
    "selfhost/tests/check_negative/unknown_type_annotation_ok.xi",
    "selfhost/tests/check_negative/unreachable_stmt.xi",
    "selfhost/tests/check_negative/var_type_mismatch.xi",
];

// ============================================================================
// Compiler runners
// ============================================================================

fn project_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    })
    .as_path()
}

fn exe_suffix() -> &'static str {
    if cfg!(target_os = "windows") { ".exe" } else { "" }
}

fn xiom_path() -> PathBuf {
    let name = format!("xiom{}", exe_suffix());
    let debug = project_root().join("target").join("debug").join(&name);
    if debug.exists() {
        return debug;
    }
    project_root().join("target").join("release").join(name)
}

fn lines_of(text: &str) -> Vec<String> {
    text.lines().map(|l| l.to_string()).collect()
}

fn tail(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

/// Lazily compile selfhost/src/main.xi once per test process into
/// target/selfhost/ and return the binary path.
fn selfhost_exe() -> &'static Path {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        let root = project_root();
        let exe = root
            .join("target")
            .join("selfhost")
            .join(format!("xiomc-self{}", exe_suffix()));
        fs::create_dir_all(exe.parent().unwrap())
            .unwrap_or_else(|e| panic!("cannot create target/selfhost: {}", e));
        let src = root.join("selfhost").join("src").join("main.xi");
        let out = Command::new(xiom_path())
            .arg("-o")
            .arg(&exe)
            .arg(&src)
            .current_dir(root)
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn xiom for the selfhost skeleton: {}", e));
        assert!(
            out.status.success(),
            "selfhost skeleton failed to compile ({}):\n{}",
            src.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        exe
    })
    .as_path()
}

/// Rust compiler IR for a corpus file. Err = compile failed.
fn rust_ir(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(xiom_path())
        .args(["--emit-ir", path])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn xiom --emit-ir: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "xiom --emit-ir exited {:?}:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stderr), 15)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

/// Selfhost compiler IR for a corpus file. Err = non-zero exit.
fn selfhost_ir(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(selfhost_exe())
        .arg(path)
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn the selfhost compiler: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "selfhost exited {:?}:\nstdout:\n{}\nstderr:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stdout), 10),
            tail(&String::from_utf8_lossy(&out.stderr), 10)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

/// Rust compiler canonical token dump (`--dump-tokens`).
fn rust_token_dump(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(xiom_path())
        .args(["--dump-tokens", path])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn xiom --dump-tokens: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "xiom --dump-tokens exited {:?}:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stderr), 10)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

/// Selfhost compiler canonical token dump (`--dump-tokens`). `lines_of`
/// strips the CRLF the XIOM CRT printf emits on Windows pipes.
fn selfhost_token_dump(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(selfhost_exe())
        .args(["--dump-tokens", path])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn selfhost --dump-tokens: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "selfhost --dump-tokens exited {:?}:\nstdout:\n{}\nstderr:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stdout), 10),
            tail(&String::from_utf8_lossy(&out.stderr), 10)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

/// Rust compiler canonical AST dump (`--dump-ast`).
fn rust_ast_dump(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(xiom_path())
        .args(["--dump-ast", path])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn xiom --dump-ast: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "xiom --dump-ast exited {:?}:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stderr), 10)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

/// Selfhost compiler canonical AST dump (`--dump-ast`).
fn selfhost_ast_dump(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(selfhost_exe())
        .args(["--dump-ast", path])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn selfhost --dump-ast: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "selfhost --dump-ast exited {:?}:\nstdout:\n{}\nstderr:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stdout), 10),
            tail(&String::from_utf8_lossy(&out.stderr), 10)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

/// Rust compiler canonical checker dump (`--dump-check`).
fn rust_check_dump(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(xiom_path())
        .args(["--dump-check", path])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn xiom --dump-check: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "xiom --dump-check exited {:?}:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stderr), 10)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

/// Selfhost compiler canonical checker dump (`--dump-check`).
fn selfhost_check_dump(path: &str) -> Result<Vec<String>, String> {
    let out = Command::new(selfhost_exe())
        .args(["--dump-check", path])
        .current_dir(project_root())
        .output()
        .map_err(|e| format!("failed to spawn selfhost --dump-check: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "selfhost --dump-check exited {:?}:\nstdout:\n{}\nstderr:\n{}",
            out.status.code(),
            tail(&String::from_utf8_lossy(&out.stdout), 10),
            tail(&String::from_utf8_lossy(&out.stderr), 10)
        ));
    }
    Ok(lines_of(&String::from_utf8_lossy(&out.stdout)))
}

// ============================================================================
// IR comparison
// ============================================================================

struct Features {
    fns: usize,
    calls: usize,
    branches: usize,
    rets: usize,
    structs: usize,
}

fn count_features(ir: &[String]) -> Features {
    Features {
        fns: ir.iter().filter(|l| l.starts_with("define ")).count(),
        calls: ir.iter().filter(|l| l.contains("call ")).count(),
        branches: ir.iter().filter(|l| l.contains(" br ")).count(),
        rets: ir.iter().filter(|l| l.starts_with("  ret ")).count(),
        structs: ir.iter().filter(|l| l.starts_with("%struct.")).count(),
    }
}

fn fn_names(ir: &[String]) -> Vec<&str> {
    ir.iter()
        .filter(|l| l.starts_with("define "))
        .filter_map(|l| l.split('@').nth(1).and_then(|s| s.split('(').next()))
        .collect()
}

/// T2 normalization: strip register/string numbering (`%tmp\d+` -> `%tmp`,
/// `@.str\d+` -> `@.str`) so both emitters are compared on structure.
fn normalize_ir(lines: &[String]) -> Vec<String> {
    lines.iter().map(|l| normalize_line(l)).collect()
}

fn normalize_line(line: &str) -> String {
    let b = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(b"%tmp") {
            out.push_str("%tmp");
            i += 4;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        } else if b[i..].starts_with(b"@.str") {
            out.push_str("@.str");
            i += 5;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        } else {
            let ch = line[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn first_diff(a: &[String], b: &[String]) -> Option<String> {
    let n = a.len().max(b.len());
    for i in 0..n {
        let x = a.get(i);
        let y = b.get(i);
        if x != y {
            let show = |v: Option<&String>| match v {
                Some(s) => format!("{:?}", s),
                None => "<missing>".to_string(),
            };
            return Some(format!(
                "first difference at line {}:\n  rust:     {}\n  selfhost: {}",
                i + 1,
                show(x),
                show(y)
            ));
        }
    }
    None
}

/// T1 gate: well-formed headers, minimum function counts, fn-name overlap.
fn gate_t1(entry: &CorpusEntry, rust: &[String], sh: &[String]) -> Result<String, String> {
    if rust.is_empty() {
        return Err("rust IR is empty".to_string());
    }
    if !rust[0].starts_with("; XIOM") {
        return Err(format!("rust IR lacks the XIOM header: {:?}", rust[0]));
    }
    if sh.is_empty() {
        return Err("selfhost IR is empty".to_string());
    }
    if !sh[0].starts_with("; XIOM") {
        return Err(format!("selfhost IR lacks the XIOM header: {:?}", sh[0]));
    }
    let rf = count_features(rust);
    let sf = count_features(sh);
    if rf.fns < entry.min_rust_fns {
        return Err(format!("rust fns {} < {}", rf.fns, entry.min_rust_fns));
    }
    if sf.fns < entry.min_self_fns {
        return Err(format!("selfhost fns {} < {}", sf.fns, entry.min_self_fns));
    }
    let rust_names = fn_names(rust);
    let sh_names = fn_names(sh);
    let missing: Vec<&str> = rust_names
        .iter()
        .copied()
        .filter(|n| !sh_names.contains(n))
        .collect();
    let total = rust_names.len();
    let matched = total - missing.len();
    let ratio = if total == 0 { 1.0 } else { matched as f64 / total as f64 };
    if ratio < entry.min_name_match {
        return Err(format!(
            "fn-name match {}/{} ({:.2}) < {}; missing: {}",
            matched,
            total,
            ratio,
            entry.min_name_match,
            missing.join(", ")
        ));
    }
    Ok(format!(
        "fns={}/{} calls={}/{} br={}/{} ret={}/{} structs={}/{} names={}/{}",
        rf.fns, sf.fns, rf.calls, sf.calls, rf.branches, sf.branches, rf.rets, sf.rets,
        rf.structs, sf.structs, matched, total
    ))
}

fn tier() -> u32 {
    std::env::var("XIOM_SELFHOST_DIFF_TIER")
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(1)
        .clamp(1, 3)
}

// ============================================================================
// Tests
// ============================================================================

/// Phase 0 gate: T1 green over the whole corpus (see the file header).
#[test]
fn diff_corpus() {
    let tier = tier();
    let entries = corpus();
    for e in &entries {
        let p = project_root().join(e.path);
        assert!(p.exists(), "corpus entry missing from the checkout: {}", e.path);
    }
    eprintln!(
        "selfhost diff corpus: {} files, tier T{} (XIOM_SELFHOST_DIFF_TIER)",
        entries.len(),
        tier
    );

    let workers = std::thread::available_parallelism()
        .map(|n| n.get().clamp(2, 4))
        .unwrap_or(2);
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, Result<String, String>)>> = Mutex::new(Vec::new());

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= entries.len() {
                    break;
                }
                let entry = &entries[i];
                let outcome = (|| -> Result<String, String> {
                    let rust = rust_ir(entry.path)?;
                    let sh = selfhost_ir(entry.path)?;
                    let summary = gate_t1(entry, &rust, &sh)?;
                    if tier >= 2 {
                        let a = normalize_ir(&rust);
                        let b = normalize_ir(&sh);
                        if let Some(d) = first_diff(&a, &b) {
                            return Err(format!("T2 normalized IR mismatch: {}", d));
                        }
                    }
                    if tier >= 3 {
                        if let Some(d) = first_diff(&rust, &sh) {
                            return Err(format!("T3 exact IR mismatch: {}", d));
                        }
                    }
                    Ok(summary)
                })();
                results.lock().unwrap().push((i, outcome));
            });
        }
    });

    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|(i, _)| *i);
    assert_eq!(results.len(), entries.len(), "worker pool dropped corpus entries");

    let mut failures: Vec<String> = Vec::new();
    for (i, outcome) in &results {
        match outcome {
            Ok(summary) => eprintln!("  {}: {}", entries[*i].path, summary),
            Err(err) => {
                eprintln!("  FAIL {}: {}", entries[*i].path, err);
                failures.push(format!("{}: {}", entries[*i].path, err));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "selfhost diff corpus (T{}): {} failure(s) of {} files:\n{}",
        tier,
        failures.len(),
        entries.len(),
        failures.join("\n")
    );
}

/// Phase 0 helper selfcheck: the pure-XIOM runtime_ffi ports are asserted
/// against the C helpers' known outputs (stdlib/runtime/xiom_runtime.c).
#[test]
fn runtime_ffi_selfcheck() {
    let out = Command::new(selfhost_exe())
        .arg("--selfcheck")
        .current_dir(project_root())
        .output()
        .expect("failed to spawn the selfhost selfcheck");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "selfhost --selfcheck exited {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        stdout,
        stderr
    );
    assert!(
        stdout.contains("SELFCHECK OK"),
        "selfhost --selfcheck did not report success:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}

/// Phase 1 gate: the selfhost lexer's canonical token dump is line-for-line
/// identical to the Rust lexer's `--dump-tokens` over the whole corpus.
///
/// Format ownership: `crates/xiom/src/main.rs::dump_tokens` and
/// `selfhost/src/lexer.xi::dump_tokens` define the same byte-stable format
/// (see the lexer header). Float payloads are dumped as the token LEXEME
/// (float VALUE parity is deferred until the selfhost has a correctly
/// rounded decimal->f64 parser / bitcast intrinsic); every other payload is
/// value-exact.
#[test]
fn diff_tokens() {
    let entries = corpus();
    eprintln!("selfhost token-dump corpus: {} files", entries.len());

    let mut failures: Vec<String> = Vec::new();
    let mut total_tokens = 0usize;
    for e in &entries {
        let outcome = (|| -> Result<usize, String> {
            let rust = rust_token_dump(e.path)?;
            let sh = selfhost_token_dump(e.path)?;
            if let Some(d) = first_diff(&rust, &sh) {
                return Err(format!("token dump mismatch: {}", d));
            }
            Ok(rust.len())
        })();
        match outcome {
            Ok(n) => {
                total_tokens += n;
                eprintln!("  {}: {} tokens", e.path, n);
            }
            Err(err) => {
                eprintln!("  FAIL {}: {}", e.path, err);
                failures.push(format!("{}: {}", e.path, err));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "selfhost token-dump parity (Phase 1): {} failure(s) of {} files:\n{}",
        failures.len(),
        entries.len(),
        failures.join("\n")
    );
    eprintln!(
        "selfhost token dump parity: {} files, {} tokens",
        entries.len(),
        total_tokens
    );
}

/// Phase 2 gate: the selfhost parser's canonical AST dump is line-for-line
/// identical to the Rust parser's `--dump-ast` over the whole corpus.
///
/// Format ownership: `crates/xiom/src/main.rs::dump_ast` (AstDump) and
/// `selfhost/src/parser.xi` + `ast_dump.xi` define the same byte-stable
/// format (see the format notes above `dump_ast`). Float literal payloads
/// dump the source LEXEME (float VALUE parity is deferred until the selfhost
/// has a correctly rounded decimal->f64 parser / bitcast intrinsic); every
/// other payload is value-exact.
///
/// Green over the whole 83-file corpus: the Phase 2 completion gate. The
/// selfhost AST is an arena (`Vec[Node]` + Int indices) because recursive
/// value enums mis-lower (COMPILER_BUGS 2026-10-02 (e)); the dump resolves
/// the arena back into the Rust tree shape.
#[test]
fn diff_ast() {
    let entries = corpus();
    eprintln!("selfhost ast-dump corpus: {} files", entries.len());

    let mut failures: Vec<String> = Vec::new();
    let mut total_nodes = 0usize;
    for e in &entries {
        let outcome = (|| -> Result<usize, String> {
            let rust = rust_ast_dump(e.path)?;
            let sh = selfhost_ast_dump(e.path)?;
            if let Some(d) = first_diff(&rust, &sh) {
                return Err(format!("ast dump mismatch: {}", d));
            }
            Ok(rust.len())
        })();
        match outcome {
            Ok(n) => {
                total_nodes += n;
                eprintln!("  {}: {} nodes", e.path, n);
            }
            Err(err) => {
                eprintln!("  FAIL {}: {}", e.path, err);
                failures.push(format!("{}: {}", e.path, err));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "selfhost ast-dump parity (Phase 2): {} failure(s) of {} files:\n{}",
        failures.len(),
        entries.len(),
        failures.join("\n")
    );
    eprintln!(
        "selfhost ast dump parity: {} files, {} nodes",
        entries.len(),
        total_nodes
    );
}

/// Phase 3 gate: the selfhost checker's canonical `--dump-check` output is
/// line-for-line identical to the Rust checker's over the whole corpus, and
/// both drivers match the committed negative/accept manifests.
///
/// Non-vacuity is enforced structurally: the corpus carries real diagnostics
/// (4x W003 on the stdlib smoke fixture, 1x W008 on m37_short_circuit) whose
/// exact lines are re-asserted here, so a pair of scripts that agree on a
/// constant `CHECK-OK` cannot pass the gate.
#[test]
fn diff_check() {
    let entries = corpus();
    eprintln!("selfhost check-dump corpus: {} files", entries.len());

    let mut failures: Vec<String> = Vec::new();
    let mut total_diags = 0usize;
    for e in &entries {
        let outcome = (|| -> Result<usize, String> {
            let rust = rust_check_dump(e.path)?;
            let sh = selfhost_check_dump(e.path)?;
            if let Some(d) = first_diff(&rust, &sh) {
                return Err(format!("check dump mismatch: {}", d));
            }
            let n = rust.iter().filter(|l| *l != "CHECK-OK").count();
            Ok(n)
        })();
        match outcome {
            Ok(n) => {
                total_diags += n;
                if n > 0 {
                    eprintln!("  {}: {} diagnostic line(s)", e.path, n);
                }
            }
            Err(err) => {
                eprintln!("  FAIL {}: {}", e.path, err);
                failures.push(format!("{}: {}", e.path, err));
            }
        }
    }

    // Non-vacuity: the two corpus entries that carry diagnostics must still
    // carry exactly those diagnostics on BOTH drivers (the Rust-vs-selfhost
    // equality above already ties them together).
    let smoke_path = "stdlib/tests/smoke/smoke_guard_fault.xi";
    match rust_check_dump(smoke_path) {
        Ok(lines) => {
            let expected: Vec<String> = ["23:3", "33:3", "43:3", "53:3"]
                .iter()
                .map(|loc| format!("warning W003 {} unreachable statement (the previous statement always exits)", loc))
                .collect();
            if lines != expected {
                failures.push(format!(
                    "{}: corpus W003 diagnostics drifted:\n{}",
                    smoke_path,
                    lines.join("\n")
                ));
            }
        }
        Err(err) => failures.push(format!("{}: {}", smoke_path, err)),
    }
    let short_path = "tests/regression/m37_short_circuit.xi";
    match selfhost_check_dump(short_path) {
        Ok(lines) => {
            let expected = vec![
                "warning W008 10:11 integer division by a zero literal always traps at runtime"
                    .to_string(),
            ];
            if lines != expected {
                failures.push(format!(
                    "{}: corpus W008 diagnostics drifted:\n{}",
                    short_path,
                    lines.join("\n")
                ));
            }
        }
        Err(err) => failures.push(format!("{}: {}", short_path, err)),
    }

    // Negative/accept manifest: `.expected` files are the source of truth for
    // BOTH drivers (a Rust-side message change must update the manifest).
    for case in NEGATIVE_CASES {
        assert!(
            project_root().join(case).exists(),
            "check_negative case missing: {}",
            case
        );
        let expected_path = project_root().join(case).with_extension("expected");
        let expected_text = match fs::read_to_string(&expected_path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("{}: cannot read .expected: {}", case, e));
                continue;
            }
        };
        let expected = lines_of(&expected_text);
        for (driver, dump) in [
            ("rust", rust_check_dump(case)),
            ("selfhost", selfhost_check_dump(case)),
        ] {
            match dump {
                Ok(lines) => {
                    if let Some(d) = first_diff(&expected, &lines) {
                        failures.push(format!("{} ({} driver): {}", case, driver, d));
                    }
                }
                Err(err) => failures.push(format!("{} ({} driver): {}", case, driver, err)),
            }
        }
    }

    assert!(
        failures.is_empty(),
        "selfhost checker parity (Phase 3): {} failure(s) over {} corpus files + {} manifest cases:\n{}",
        failures.len(),
        entries.len(),
        NEGATIVE_CASES.len(),
        failures.join("\n")
    );
    eprintln!(
        "selfhost check parity: {} files ({} diagnostic lines, non-vacuous) + {} manifest cases",
        entries.len(),
        total_diags,
        NEGATIVE_CASES.len()
    );
}
