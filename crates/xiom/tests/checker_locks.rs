// Checker locks for the stdlib-lane relay findings (2026-09-24).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//   m135 -- CLAUSE-position Bool-vs-Int comparison must be REJECTED
//          (it used to compile and coerce silently);
//   m136 -- well-typed clauses (implicit self, @pre, result) stay green.

use std::path::PathBuf;
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join(name)
        .join("main.xi")
}

fn run_on(name: &str) -> (String, Option<i32>, PathBuf) {
    let exe = std::env::temp_dir().join(format!(
        "xiom_check_{}_{}{}",
        name,
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg(fixture(name))
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
        exe,
    )
}

/// Stage 6 tier 3: compile with the opt-in `--lint-strict` flag. Same
/// contract as `run_on` plus the flag; the caller owns the executable.
fn run_strict_on(name: &str) -> (String, Option<i32>, PathBuf) {
    let exe = std::env::temp_dir().join(format!(
        "xiom_check_strict_{}_{}{}",
        name,
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg("--lint-strict")
        .arg(fixture(name))
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
        exe,
    )
}

/// m150 (R-5 benchmark relay): `--check`-only helper. The R-5 false positive
/// was a check-phase finding, and its full relay shape currently stops at
/// codegen on a separate pre-existing C001, so the positive lock must not
/// link.
fn check_on(name: &str) -> (String, Option<i32>) {
    let output = Command::new(xiom_bin())
        .arg("--check")
        .arg(fixture(name))
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
    )
}

/// m259 (REL-20261010-1900): a body whose resolution already failed (T001
/// undefined variable) must not get W004 reachability cascades from a
/// bare-variant match over an unresolved enum scrutinee -- rustc gates
/// `check_match` behind `Result<(), ErrorGuaranteed>`; we gate on body error
/// taint + an error-typed scrutinee (checked in `check_fn_decl` /
/// `error_with_cause` / both W004 call sites).
#[test]
fn m259_w004_suppressed_on_error_tainted_body() {
    let (stderr, code) = check_on("m259_w004_no_cascade");
    assert_ne!(
        code,
        Some(0),
        "the resolution error must fail the check. stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("T001"),
        "the real T001 resolution error must still be reported. stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("W004"),
        "an error-tainted body must not emit W004 cascades. stderr:\n{stderr}"
    );
}

/// m266 (REL-20261010-2124-xvector + REL-20261010-1802 gate): an unresolved
/// `use` is a hard, SINGLE error at the import line. Pre-fix the phantom
/// import was ignored and the program either PASSED with bogus W004
/// unreachable-arm warnings or failed with a cross-module T001 ("pattern
/// 'Corruption' belongs to 'DbError'") once a same-named twin existed.
#[test]
fn m266_unresolved_import_one_clear_error() {
    let (stderr, code) = check_on("m266_unresolved_import");
    assert_ne!(
        code,
        Some(0),
        "the unresolved import must fail the check. stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("unresolved import 'xiom.core.error_does_not_exist'"),
        "the import line must carry the error. stderr:\n{stderr}"
    );
    let t001 = stderr.matches("error[T001]").count();
    assert_eq!(
        t001, 1,
        "exactly ONE clear error expected (no cascades). stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("belongs to"),
        "no phantom cross-module pattern error. stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("W004"),
        "no phantom W004 unreachable-arm warnings. stderr:\n{stderr}"
    );
}

#[test]
fn m141_mixed_brackets_lax_then_strict() {
    // LAX default (held for v0.62.0: the pin carries 3 stdlib mixed sites;
    // the flip lands with the next release after the stdlib wave fixes
    // `xiom/io/fs.xi` x2 + `xiom/math/algebra_extended`): the legacy mixed
    // spelling still compiles+run.
    let (stderr, code, exe) = run_on("m141_mixed_bracket_switch");
    assert_eq!(code, Some(0), "lax default must accept the mixed spelling. stderr:\n{stderr}");
    let run = Command::new(&exe).output().expect("run m141");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "m141 must exit 0");

    // STRICT opt-in: the closer must match the opener -> P001.
    let exe = std::env::temp_dir().join(format!(
        "xiom_brackets_strict_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg(fixture("m141_mixed_bracket_switch"))
        .arg("-o")
        .arg(&exe)
        .env("XIOM_STRICT_BRACKETS", "1")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let _ = std::fs::remove_file(&exe);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "strict mode must reject. stderr:\n{stderr}");
    assert!(
        stderr.contains("expected '>'") || stderr.contains("expected ']'"),
        "expected a bracket P001 diagnostic, got:\n{stderr}"
    );
}

#[test]
fn m135_clause_bool_mix_is_rejected() {
    let (stderr, code, exe) = run_on("m135_clause_bool_int");
    let _ = std::fs::remove_file(&exe);
    assert_ne!(code, Some(0), "clause Bool/Int comparison must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("contract clause compares Bool"),
        "expected the clause Bool-mix diagnostic, got:\n{stderr}"
    );
}

#[test]
fn m136_well_typed_clauses_still_run() {
    let (stderr, code, exe) = run_on("m136_well_typed_clauses");
    assert_eq!(code, Some(0), "well-typed clauses must compile. stderr:\n{stderr}");
    let run = Command::new(&exe).status().expect("run m136");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.code(), Some(0), "m136 must exit 0");
}

#[test]
fn m137_strict_clause_mode_rejects_non_bool_predicates() {
    // Legacy LIGHT opt-out: `requires: x` (Int) is accepted.
    let exe = std::env::temp_dir().join(format!(
        "xiom_clause_light_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg(fixture("m137_clause_non_bool"))
        .arg("-o")
        .arg(&exe)
        .env("XIOM_STRICT_CLAUSES", "0")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let _ = std::fs::remove_file(&exe);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "light opt-out must accept the fixture. stderr:\n{stderr}");

    // RELEASE DEFAULT (v0.62.0): the full predicate rule rejects it (the
    // stdlib lane verified their main + the pinned corpus under strict).
    let exe = std::env::temp_dir().join(format!(
        "xiom_clause_strict_{}{}",
        std::process::id(),
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let output = Command::new(xiom_bin())
        .arg(fixture("m137_clause_non_bool"))
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let _ = std::fs::remove_file(&exe);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "the strict default must reject. stderr:\n{stderr}");
    assert!(
        stderr.contains("contract clause must be Bool"),
        "expected the strict predicate diagnostic, got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Stage 6 lint wave (W002/W003): warning-only, user-program scope. Each lint
// gets a positive fixture (code in stderr AND compile exit 0) and a negative
// fixture that must stay silent.
// ---------------------------------------------------------------------------

#[test]
fn m151_w002_unconditional_cycle_warns() {
    let (stderr, code, exe) = run_on("m151_w002_cycle");
    let _ = std::fs::remove_file(&exe);
    // Compile exit 0: the lint is warning-only and must never block a build.
    // (The fixture is deliberately NOT executed -- the cycle cannot
    // terminate.)
    assert_eq!(code, Some(0), "W002 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W002]"),
        "expected warning[W002] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unconditional recursive cycle"),
        "expected the cycle message, got:\n{stderr}"
    );
}

#[test]
fn m150_dl_num_parse_checks_clean() {
    // R-5 (benchmark relay): `use xiom.num;` + `use xiom.ffi.dl;` used to trip
    // the global extern-name gate on xiom.math.primitives' own `pub fn abs`
    // (the libc `abs` extern lives in xiom.ffi.c, pulled in through the
    // xiom.ffi parent module). The relay's exact program must pass the
    // checker with no catalog-body finding.
    let (stderr, code) = check_on("m150_dl_num_parse");
    assert_eq!(code, Some(0), "num + ffi.dl must pass --check. stderr:\n{stderr}");
    assert!(
        !stderr.contains("catalog body [xiom.math.primitives]"),
        "the math.primitives catalog body must not produce a finding:\n{stderr}"
    );
}

#[test]
fn m151_w002_guard_stays_silent() {
    let (stderr, code, exe) = run_on("m151_w002_guard");
    assert_eq!(code, Some(0), "guard-first recursion must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W002]"),
        "guard-first recursion must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m151_w002_guard");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the guard fixture must run cleanly");
}

#[test]
fn m151_w003_unreachable_warns() {
    let (stderr, code, exe) = run_on("m151_w003_unreachable");
    assert_eq!(code, Some(0), "W003 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W003]"),
        "expected warning[W003] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unreachable statement"),
        "expected the unreachable-statement message, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m151_w003_unreachable");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the W003 fixture must still run cleanly");
}

#[test]
fn m151_w003_guarded_stays_silent() {
    let (stderr, code, exe) = run_on("m151_w003_guarded");
    assert_eq!(code, Some(0), "guarded fixture must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W003]"),
        "guarded shapes must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m151_w003_guarded");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the guarded fixture must run cleanly");
}

#[test]
fn m154_w004_unreachable_arm_warns() {
    let (stderr, code, exe) = run_on("m154_w004_unreachable");
    assert_eq!(code, Some(0), "W004 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W004]"),
        "expected warning[W004] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unreachable match arm"),
        "expected the unreachable-arm message, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m154_w004_unreachable");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the W004 fixture must run cleanly");
}

#[test]
fn m154_w004_guarded_stays_silent() {
    let (stderr, code, exe) = run_on("m154_w004_guard");
    assert_eq!(code, Some(0), "guarded fixture must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W004]"),
        "guarded duplicates/catch-alls must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m154_w004_guard");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the guarded fixture must run cleanly");
}

// XVC-C-03 (m220): bare unit-enum variant patterns are specific variants,
// not bindings -- distinct bare arms must not warn.
#[test]
fn m220_w004_bare_variants_silent() {
    let (stderr, code, exe) = run_on("m220_w004_bare_variants");
    assert_eq!(code, Some(0), "bare unit-enum match must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W004]"),
        "distinct BARE unit-enum arms must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m220_w004_bare_variants");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the bare-variant fixture must run cleanly");
}

// ---------------------------------------------------------------------------
// Stage 6 lint wave, tier 2: W006 (shift amount out of range), W007
// (self-comparison always true/false), W008 (literal integer div/rem by
// zero). Same contract: warning-only, compile exit 0, negatives silent.
// The W008/W006 POSITIVES are compile-only -- their runtime shapes trap /
// produce garbage by design (see the fixture headers).
// ---------------------------------------------------------------------------

#[test]
fn m156_w008_div_by_zero_warns() {
    let (stderr, code, exe) = run_on("m156_w008_div_zero");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W008 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W008]"),
        "expected warning[W008] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("zero literal always traps"),
        "expected the div/rem-by-zero message, got:\n{stderr}"
    );
}

#[test]
fn m156_w008_guard_stays_silent() {
    let (stderr, code, exe) = run_on("m156_w008_guard");
    assert_eq!(code, Some(0), "float/non-literal divisors must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W008]"),
        "float and non-literal divisors must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m156_w008_guard");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the W008 guard fixture must run cleanly");
}

#[test]
fn m157_w006_shift_out_of_range_warns() {
    let (stderr, code, exe) = run_on("m157_w006_shift");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W006 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W006]"),
        "expected warning[W006] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("out of range"),
        "expected the shift-range message, got:\n{stderr}"
    );
}

#[test]
fn m157_w006_guard_stays_silent() {
    let (stderr, code, exe) = run_on("m157_w006_guard");
    assert_eq!(code, Some(0), "in-range shifts must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W006]"),
        "in-range and variable shifts must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m157_w006_guard");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the W006 guard fixture must run cleanly");
}

#[test]
fn m158_w007_self_comparison_warns() {
    let (stderr, code, exe) = run_on("m158_w007_selfcmp");
    assert_eq!(code, Some(0), "W007 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W007]"),
        "expected warning[W007] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("self-comparison is always"),
        "expected the self-comparison message, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m158_w007_selfcmp");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the W007 fixture must run cleanly");
}

#[test]
fn m158_w007_guard_stays_silent() {
    let (stderr, code, exe) = run_on("m158_w007_guard");
    assert_eq!(code, Some(0), "float/distinct/call comparisons must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W007]"),
        "float, distinct and call comparisons must stay silent, got:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m158_w007_guard");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the W007 guard fixture must run cleanly");
}

// ---------------------------------------------------------------------------
// R-2c / R-2 partial (benchmark relay): unwrap-chain Vec.len receiver
// resolution and inline aggregate match-payload aliasing.
// ---------------------------------------------------------------------------

#[test]
fn m159_r2c_unwrap_vec_len_runs() {
    let (stderr, code, exe) = run_on("m159_r2c_unwrap_vec_len");
    assert_eq!(code, Some(0), "R-2c fixture must compile. stderr:\n{stderr}");
    let run = Command::new(&exe).output().expect("run m159_r2c_unwrap_vec_len");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "R-2c fixture must run cleanly");
    assert!(
        String::from_utf8_lossy(&run.stdout).contains('1'),
        "expected the pushed payload len (1) on stdout, got:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
}

#[test]
fn m160_r2_match_alias_runs() {
    let (stderr, code, exe) = run_on("m160_r2_match_alias");
    assert_eq!(code, Some(0), "R-2 fixture must compile. stderr:\n{stderr}");
    let run = Command::new(&exe).output().expect("run m160_r2_match_alias");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "R-2 fixture must run cleanly");
}

#[test]
fn m161_byte_at_direct_compare_runs() {
    let (stderr, code, exe) = run_on("m161_byte_at_direct_compare");
    assert_eq!(code, Some(0), "byte_at fixture must compile. stderr:\n{stderr}");
    let run = Command::new(&exe).output().expect("run m161_byte_at_direct_compare");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "direct byte_at compares must be correct");
}

#[test]
fn m162_sameleaf_fn_does_not_poison_catalog_bodies() {
    // Playground relay: a user module's same-leaf fn (`char_at`) used to
    // poison the catalog-body CHECK of xiom.num (bogus T001s from the wrong
    // return type) and the CODEGEN binding of num's `use
    // xiom.string.char_at` (runtime "invalid index"). The checker now
    // prefers the body's explicit item import and records the resolved
    // target for codegen.
    let (stderr, code, exe) = run_on("m162_sameleaf_catalog_poison");
    assert_eq!(
        code,
        Some(0),
        "a same-leaf user fn must not poison catalog bodies (m162). stderr:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m162_sameleaf_catalog_poison");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(
        run.status.code(),
        Some(0),
        "catalog-body calls must bind their own imports at runtime (m162)"
    );
}

#[test]
fn m155_r2d_check_library_file_passes() {
    // R-2d: `--check` must NOT implicit-main-wrap a library file (top-level
    // declarations, no `fn main`); wrapping produced a bogus P001 at the
    // first `requires:` clause while a full compile succeeded.
    let output = Command::new(xiom_bin())
        .arg("--check")
        .arg(fixture("m155_r2d_check_library"))
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "a library-style --check must pass (R-2d); stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("Type check PASSED"),
        "expected the pass line, got:\n{stderr}"
    );
}

#[test]
fn m150_genuine_extern_still_gated() {
    // R-5 fix guard: only a fn DEFINITION shadows a same-named extern mark,
    // so a plain `extern "C"` call in safe user code still requires `unsafe`.
    let (stderr, code, exe) = run_on("m150_extern_gate");
    let _ = std::fs::remove_file(&exe);
    assert_ne!(code, Some(0), "extern call without unsafe must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("requires an `unsafe` block"),
        "expected the extern-confinement diagnostic, got:\n{stderr}"
    );
}

#[test]
fn m171_qualified_variant_patterns_are_exhaustive() {
    // Selfhost Phase 2 finding (f): "Tree.Leaf"/"Tree.Node" must count as
    // covered; both arms used to be flagged W000 non-exhaustive.
    let (stderr, code) = check_on("m171_qualified_variant_pattern");
    assert_eq!(code, Some(0), "qualified patterns must check clean. stderr:\n{stderr}");
    assert!(
        !stderr.contains("non-exhaustive"),
        "qualified variants must count as covered, got:\n{stderr}"
    );
}

#[test]
fn m172_enum_aggregate_equality_rejected() {
    // Selfhost Phase 2 finding (g): aggregate payloads cannot lower to a
    // derived equality -- must be a diagnostic, not `icmp %struct.Vec`.
    let (stderr, code) = check_on("m172_enum_eq_aggregate_rejected");
    assert_ne!(code, Some(0), "aggregate-payload enum equality must be rejected. stderr:\n{stderr}");
    assert!(
        stderr.contains("non-comparable aggregates"),
        "expected the aggregate-payload diagnostic, got:\n{stderr}"
    );
}

#[test]
fn m173_scalar_payload_enum_equality_still_green() {
    // Guard for (g): scalar/Str payloads keep compiling and comparing.
    let (stderr, code, exe) = run_on("m173_enum_eq_scalar_payload");
    assert_eq!(code, Some(0), "scalar/Str payload equality must keep compiling. stderr:\n{stderr}");
    let run = Command::new(&exe).output().expect("run m173");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "m173 runtime checks failed");
}

#[test]
fn m178_result_patterns_on_non_result_rejected() {
    // Wave-57 (stdlib relay): `match str_call() { Ok(v) => ..., Err(_) => ... }`
    // used to pass and produce invalid IR (stale local slot / dominance).
    let (stderr, code) = check_on("m178_illtyped_match_rejected");
    assert_ne!(code, Some(0), "ill-typed match must be rejected. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot match 'Str'"),
        "expected the pattern/type mismatch diagnostic, got:\n{stderr}"
    );
}

#[test]
fn m177_result_payload_mismatch_rejected() {
    // Packages relay (consul): Result[Bool, Str] used to satisfy a declared
    // Result[Int, Str] (Bool counted as numeric in generic args) and
    // .unwrap() read garbage. Must now be a return-type mismatch.
    let (stderr, code) = check_on("m177_result_payload_mismatch");
    assert_ne!(code, Some(0), "Result payload mismatch must be rejected. stderr:\n{stderr}");
    assert!(
        stderr.contains("expected Result[Int, Str], found Result[Bool, Str]"),
        "expected the return-type mismatch diagnostic, got:\n{stderr}"
    );
}

// m252 (Stage 6 item 1, STAGE6_PERF_PLAN): persistent catalog-body CHECK
// cache. Locks: (a) cold vs warm `--check` diagnostics byte-identical after
// stripping the [timings] probe lines, warm run reports hits; (b) cold vs
// warm `--emit-ir` stdout byte-identical -- the replayed
// `catalog_resolved_calls` delta must reproduce codegen's cross-module
// binding exactly; (c) a sibling-module edit invalidates the cached body and
// surfaces the new diagnostic.
fn m252_fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m252_body_check_cache")
}

fn m252_strip_timings(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .filter(|l| !l.contains("[timings]"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn m252_cache_stats(stderr: &[u8]) -> (usize, usize) {
    for line in String::from_utf8_lossy(stderr).lines() {
        if let Some(rest) = line.split("body-cache hits=").nth(1) {
            let mut parts = rest.split(" misses=");
            let hits = parts.next().and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
            let misses = parts.next().and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
            return (hits, misses);
        }
    }
    (0, 0)
}

#[test]
fn m252_body_check_cache_parity_and_invalidate() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = std::env::temp_dir().join(format!("xiom_m252_{}_{nanos:x}", std::process::id()));
    let proj = root.join("proj");
    let home = root.join("home");
    std::fs::create_dir_all(&proj).expect("create m252 project dir");
    std::fs::create_dir_all(&home).expect("create m252 home dir");
    for name in ["main.xi", "m252_helper.xi", "m252_math.xi"] {
        std::fs::copy(m252_fixture_dir().join(name), proj.join(name))
            .unwrap_or_else(|e| panic!("copy {name}: {e}"));
    }
    let main = proj.join("main.xi");
    let run = |args: &[&str]| {
        Command::new(xiom_bin())
            .args(args)
            .arg(&main)
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("XIOM_TIMINGS", "1")
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()))
    };

    // (a) cold vs warm --check: identical diagnostics; warm hits.
    let c1 = run(&["--check"]);
    assert_eq!(
        c1.status.code(),
        Some(0),
        "cold check must pass:\n{}",
        String::from_utf8_lossy(&c1.stderr)
    );
    let (_, cold_misses) = m252_cache_stats(&c1.stderr);
    assert!(
        cold_misses >= 1,
        "cold check must record body-cache misses:\n{}",
        String::from_utf8_lossy(&c1.stderr)
    );
    let c2 = run(&["--check"]);
    assert_eq!(
        c2.status.code(),
        Some(0),
        "warm check must pass:\n{}",
        String::from_utf8_lossy(&c2.stderr)
    );
    let (warm_hits, _) = m252_cache_stats(&c2.stderr);
    assert!(
        warm_hits >= 1,
        "warm check must hit the body cache:\n{}",
        String::from_utf8_lossy(&c2.stderr)
    );
    assert_eq!(
        m252_strip_timings(&c1.stderr),
        m252_strip_timings(&c2.stderr),
        "cold vs warm diagnostics must be byte-identical"
    );

    // (b) cold vs warm --emit-ir: stdout byte-identical (replay lock).
    let ir_home = root.join("ir_home");
    std::fs::create_dir_all(&ir_home).expect("create m252 ir home");
    let ir_run = || {
        Command::new(xiom_bin())
            .arg("--emit-ir")
            .arg(&main)
            .env("HOME", &ir_home)
            .env("USERPROFILE", &ir_home)
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()))
    };
    let ir1 = ir_run();
    let ir2 = ir_run();
    assert_eq!(
        ir1.status.code(),
        Some(0),
        "cold emit-ir must succeed:\n{}",
        String::from_utf8_lossy(&ir1.stderr)
    );
    assert_eq!(
        ir2.status.code(),
        Some(0),
        "warm emit-ir must succeed:\n{}",
        String::from_utf8_lossy(&ir2.stderr)
    );
    assert!(!ir1.stdout.is_empty(), "cold emit-ir must produce IR on stdout");
    assert_eq!(
        ir1.stdout, ir2.stdout,
        "cold vs warm IR must be byte-identical (resolved-call replay)"
    );

    // (c) invalidation: the sibling edit is re-checked and surfaces.
    std::fs::write(
        proj.join("m252_helper.xi"),
        "module m252_helper;\n\nuse m252_math.sq;\n\npub fn triple(x: Int) -> Int {\n  return sq(x) * 3 + missing_fn_xyz(x);\n}\n",
    )
    .expect("rewrite m252 helper");
    let c3 = run(&["--check"]);
    let err3 = String::from_utf8_lossy(&c3.stderr).into_owned();
    assert_ne!(c3.status.code(), Some(0), "edited helper must fail the check:\n{err3}");
    assert!(
        err3.contains("missing_fn_xyz"),
        "the post-edit diagnostic must surface:\n{err3}"
    );
    let (_, invalidated_misses) = m252_cache_stats(&c3.stderr);
    assert!(
        invalidated_misses >= 1,
        "edited source must invalidate its cached body:\n{err3}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// m285 (REL-20261010-1549-stdlib-5): shape-mismatched &Vec argument.
// ---------------------------------------------------------------------------

/// m285: passing a flat &Vec[Float64] where &Vec[Vec[Float64]] is declared
/// used to pass the checker -- `type_arg_compatible`'s one-container branch
/// returned true for ANY non-container vs container (the scalar `Float64`
/// counted as an erased `Vec[Float64]`) -- and the callee's nested element
/// read AVed at runtime (0xC0000005). Must now be a call-site T001.
#[test]
fn m285_vec_shape_arg_mismatch_rejected() {
    let (stderr, code) = check_on("m285_vec_shape_arg_mismatch");
    assert_ne!(
        code,
        Some(0),
        "the shape-mismatched &Vec argument must fail the check. stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("argument 2 type mismatch: expected Vec[Vec[Float64]], found Vec[Float64]"),
        "expected the call-site argument mismatch, got:\n{stderr}"
    );
}

/// m285 positive control: the MATCHED shape (both arguments
/// &Vec[Vec[Float64]]) must keep compiling and running.
#[test]
fn m285_vec_shape_arg_matched_still_runs() {
    let (stderr, code, exe) = run_on("m285_vec_shape_arg_matched");
    assert_eq!(code, Some(0), "the matched shape must compile. stderr:\n{stderr}");
    let run = Command::new(&exe).output().expect("run m285_vec_shape_arg_matched");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(run.status.code(), Some(0), "the matched shape must run cleanly");
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("ok"),
        "the 1x2 * 2x1 product must be [[11.0]], got:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
}

/// m291 (REL-20261010-1548-stdlib-7 enforcement half): a directly violated
/// `requires` must abort at RUNTIME. The m283 contract re-entrancy guard
/// skipped EVERY clause check (skip test `d > 0` while the counter is
/// incremented at function entry, so d == 1 at the first clause site):
/// enforcement was silently dead and this program returned 0. The guard now
/// skips only re-entrant evaluations (d > 1).
#[test]
fn m291_contract_enforcement_live() {
    let (stderr, code, exe) = run_on("m291_contract_enforcement");
    assert_eq!(
        code,
        Some(0),
        "the program must COMPILE -- the violation is runtime-only. stderr:\n{stderr}"
    );
    let run = Command::new(&exe).output().expect("run m291_contract_enforcement");
    let _ = std::fs::remove_file(&exe);
    assert_ne!(
        run.status.code(),
        Some(0),
        "a violated requires must abort at runtime (m291 guard regression would let it return 0)"
    );
    let out = String::from_utf8_lossy(&run.stdout);
    assert!(
        out.contains("contract violated: requires"),
        "expected the violation message, got:\n{out}"
    );
}

/// m346 (REL-20261016-1541-xvector / XVC-C-18): field access on a receiver
/// with type args returned the RAW declared field type ("Map[Str,
/// JsonValue]"."keys" stayed "Vec[K]"), so the inferred local bound from
/// `x.field[i]` typed as the un-substituted param K and `name.len()` was a
/// false error[T001] ("cannot call 'len' on this expression") while codegen
/// silently produced pointer-as-Int garbage. The fixture's two arms were red
/// at base d4fdd7e7 (T001 47:8 param-rooted, 64:21 payload-rooted); the
/// receiver-arg substitution makes BOTH arms resolve here. The param-rooted
/// arm's runtime VALUES remain G2 (see docs/COMPILER_BUGS.md).
#[test]
fn m346_field_chain_elem_binding_check() {
    let (stderr, code) = check_on("m346_field_chain_elem_binding");
    assert_eq!(
        code,
        Some(0),
        "the generic-substituted field chain must type-check (`name.len()` on \
         the inferred binding must resolve). stderr:\n{stderr}"
    );
}

/// m349 (REL-20261016-2004-packages / XARROW-ARRAYNAME): the legacy
/// bare-`Array` builtin method fallback captured a REGISTERED user type of
/// the same name -- `a.len()` on this struct type-checked as the builtin
/// Int (pre-fix `--check` returned 0) while codegen later miscompiled the
/// call. The fallback is now gated on `Array` not being a registered
/// user/package type; the unknown-method diagnostic must surface. Fixed
/// arrays (`Array[<elem>]`) keep their own arms and are unaffected.
#[test]
fn m349_array_builtin_method_gate() {
    let (stderr, code) = check_on("m349_array_builtin_gate");
    assert_ne!(
        code,
        Some(0),
        "`.len()` on a user struct named `Array` must fail the check. stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("cannot call 'len' on this expression"),
        "expected the unknown-method T001, got:\n{stderr}"
    );
}
/// REL-20261028-0052-compiler: a top-level function whose name is emitted as
/// a plain LLVM symbol and collides with a libc/Winsock export must warn
/// (link would fail with LNK2005 / duplicate symbol). Warning-only.
#[test]
fn m0052_reserved_system_symbol_warns() {
    let (stderr, code) = check_on("m0052_reserved_symbol_warn");
    assert_eq!(
        code,
        Some(0),
        "W009 must not block the build. stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("warning[W009]"),
        "expected warning[W009] for `fn recv`, got:\n{stderr}"
    );
    assert!(
        stderr.contains("libc/Winsock"),
        "expected the collision explanation, got:\n{stderr}"
    );
}

#[test]
fn m0052_reserved_system_symbol_silent_for_normal_names() {
    let (stderr, code) = check_on("m0052_reserved_symbol_ok");
    assert_eq!(code, Some(0), "the control fixture must check cleanly. stderr:\n{stderr}");
    assert!(
        !stderr.contains("warning[W009]"),
        "`recv_bytes` must not trigger W009, got:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Stage 6 tier 3 (--lint-strict, owner decision 2026-11-02): opt-in,
// warning-only. Each test proves the positive fixture warns ONLY under the
// flag, the default build stays silent, and the negative fixture stays silent
// under the flag. Every compile exits 0.
// ---------------------------------------------------------------------------

#[test]
fn m390_w015_unused_local_is_flag_gated() {
    let (stderr, code, exe) = run_strict_on("m390_w015_unused_local");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W015 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W015]"),
        "expected warning[W015] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unused local binding"),
        "expected the unused-local message, got:\n{stderr}"
    );

    let (default_stderr, default_code, default_exe) = run_on("m390_w015_unused_local");
    let _ = std::fs::remove_file(&default_exe);
    assert_eq!(default_code, Some(0), "default build must succeed. stderr:\n{default_stderr}");
    assert!(
        !default_stderr.contains("warning[W015]"),
        "tier 3 is opt-in: default builds must stay silent, got:\n{default_stderr}"
    );

    let (guard_stderr, guard_code, guard_exe) = run_strict_on("m390_w015_guard");
    let _ = std::fs::remove_file(&guard_exe);
    assert_eq!(guard_code, Some(0), "guard fixture must compile. stderr:\n{guard_stderr}");
    assert!(
        !guard_stderr.contains("warning[W015]"),
        "used/_prefixed locals must stay silent, got:\n{guard_stderr}"
    );
}

#[test]
fn m391_w010_unused_param_is_flag_gated() {
    let (stderr, code, exe) = run_strict_on("m391_w010_unused_param");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W010 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W010]"),
        "expected warning[W010] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unused function parameter"),
        "expected the unused-parameter message, got:\n{stderr}"
    );

    let (default_stderr, default_code, default_exe) = run_on("m391_w010_unused_param");
    let _ = std::fs::remove_file(&default_exe);
    assert_eq!(default_code, Some(0), "default build must succeed. stderr:\n{default_stderr}");
    assert!(
        !default_stderr.contains("warning[W010]"),
        "tier 3 is opt-in: default builds must stay silent, got:\n{default_stderr}"
    );

    let (guard_stderr, guard_code, guard_exe) = run_strict_on("m391_w010_guard");
    let _ = std::fs::remove_file(&guard_exe);
    assert_eq!(guard_code, Some(0), "guard fixture must compile. stderr:\n{guard_stderr}");
    assert!(
        !guard_stderr.contains("warning[W010]"),
        "_prefixed params and method params must stay silent, got:\n{guard_stderr}"
    );
}

#[test]
fn m392_w011_unused_import_is_flag_gated() {
    let (stderr, code, exe) = run_strict_on("m392_w011_unused_import");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W011 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W011]"),
        "expected warning[W011] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unused import"),
        "expected the unused-import message, got:\n{stderr}"
    );

    let (default_stderr, default_code, default_exe) = run_on("m392_w011_unused_import");
    let _ = std::fs::remove_file(&default_exe);
    assert_eq!(default_code, Some(0), "default build must succeed. stderr:\n{default_stderr}");
    assert!(
        !default_stderr.contains("warning[W011]"),
        "tier 3 is opt-in: default builds must stay silent, got:\n{default_stderr}"
    );

    let (guard_stderr, guard_code, guard_exe) = run_strict_on("m392_w011_guard");
    let _ = std::fs::remove_file(&guard_exe);
    assert_eq!(guard_code, Some(0), "guard fixture must compile. stderr:\n{guard_stderr}");
    assert!(
        !guard_stderr.contains("warning[W011]"),
        "referenced imports must stay silent, got:\n{guard_stderr}"
    );
}

#[test]
fn m393_w012_constant_condition_is_flag_gated() {
    let (stderr, code, exe) = run_strict_on("m393_w012_constant_cond");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W012 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W012]"),
        "expected warning[W012] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("constant condition"),
        "expected the constant-condition message, got:\n{stderr}"
    );

    let (default_stderr, default_code, default_exe) = run_on("m393_w012_constant_cond");
    let _ = std::fs::remove_file(&default_exe);
    assert_eq!(default_code, Some(0), "default build must succeed. stderr:\n{default_stderr}");
    assert!(
        !default_stderr.contains("warning[W012]"),
        "tier 3 is opt-in: default builds must stay silent, got:\n{default_stderr}"
    );

    let (guard_stderr, guard_code, guard_exe) = run_strict_on("m393_w012_guard");
    let _ = std::fs::remove_file(&guard_exe);
    assert_eq!(guard_code, Some(0), "guard fixture must compile. stderr:\n{guard_stderr}");
    assert!(
        !guard_stderr.contains("warning[W012]"),
        "variable conditions and `while true` must stay silent, got:\n{guard_stderr}"
    );
}

#[test]
fn m394_w013_self_assignment_is_flag_gated() {
    let (stderr, code, exe) = run_strict_on("m394_w013_self_assign");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W013 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W013]"),
        "expected warning[W013] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("self-assignment"),
        "expected the self-assignment message, got:\n{stderr}"
    );

    let (default_stderr, default_code, default_exe) = run_on("m394_w013_self_assign");
    let _ = std::fs::remove_file(&default_exe);
    assert_eq!(default_code, Some(0), "default build must succeed. stderr:\n{default_stderr}");
    assert!(
        !default_stderr.contains("warning[W013]"),
        "tier 3 is opt-in: default builds must stay silent, got:\n{default_stderr}"
    );

    let (guard_stderr, guard_code, guard_exe) = run_strict_on("m394_w013_guard");
    let _ = std::fs::remove_file(&guard_exe);
    assert_eq!(guard_code, Some(0), "guard fixture must compile. stderr:\n{guard_stderr}");
    assert!(
        !guard_stderr.contains("warning[W013]"),
        "distinct places and computed updates must stay silent, got:\n{guard_stderr}"
    );
}

#[test]
fn m395_w014_vacuous_arith_is_flag_gated() {
    let (stderr, code, exe) = run_strict_on("m395_w014_vacuous_arith");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "W014 must not block the build. stderr:\n{stderr}");
    assert!(
        stderr.contains("warning[W014]"),
        "expected warning[W014] in stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("vacuous integer arithmetic"),
        "expected the vacuous-arithmetic message, got:\n{stderr}"
    );

    let (default_stderr, default_code, default_exe) = run_on("m395_w014_vacuous_arith");
    let _ = std::fs::remove_file(&default_exe);
    assert_eq!(default_code, Some(0), "default build must succeed. stderr:\n{default_stderr}");
    assert!(
        !default_stderr.contains("warning[W014]"),
        "tier 3 is opt-in: default builds must stay silent, got:\n{default_stderr}"
    );

    let (guard_stderr, guard_code, guard_exe) = run_strict_on("m395_w014_guard");
    let _ = std::fs::remove_file(&guard_exe);
    assert_eq!(guard_code, Some(0), "guard fixture must compile. stderr:\n{guard_stderr}");
    assert!(
        !guard_stderr.contains("warning[W014]"),
        "asymmetric literals and float operands must stay silent, got:\n{guard_stderr}"
    );
}

/// m396 (SPEC.md: "`let` cannot be reassigned. `var` can."): a `let` local is
/// immutable, including the compound-assignment desugaring. This is the
/// owner's exact repro plus `+=`.
#[test]
fn m396_let_reassignment_is_hard_error() {
    let (stderr, code) = check_on("m396_let_reassign_negative");
    assert_ne!(code, Some(0), "let reassignment must fail --check. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'name'"),
        "expected the immutable-let message for 'name', got:\n{stderr}"
    );
    assert!(
        stderr.contains("(declared with 'let'; use 'var')"),
        "expected the use-'var' hint, got:\n{stderr}"
    );
    assert!(
        stderr.contains("cannot assign to immutable binding 'n'"),
        "compound assignment must get the same check, got:\n{stderr}"
    );
}

/// m396: a field/index store whose ROOT is a `let` binding is rejected.
#[test]
fn m396_let_field_store_is_hard_error() {
    let (stderr, code) = check_on("m396_let_field_store_negative");
    assert_ne!(code, Some(0), "field store through let must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'p'"),
        "expected the immutable-let message for 'p', got:\n{stderr}"
    );
}

/// m396: module-level `const` (and module `const`) cannot be assigned.
#[test]
fn m396_const_store_is_hard_error() {
    let (stderr, code) = check_on("m396_const_store_negative");
    assert_ne!(code, Some(0), "const store must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable const 'LIMIT'"),
        "expected the immutable-const message, got:\n{stderr}"
    );
}

/// m396: `let (a, b)` bindings are immutable (only `var (a, b)` is assignable).
#[test]
fn m396_destructure_let_is_hard_error() {
    let (stderr, code) = check_on("m396_destructure_let_negative");
    assert_ne!(code, Some(0), "destructured let store must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'a'"),
        "expected the immutable-let message for 'a', got:\n{stderr}"
    );
}

/// m396: match/if-let/`is` pattern bindings are immutable; the pattern
/// grammar has no explicit `var` form.
#[test]
fn m396_match_binding_is_hard_error() {
    let (stderr, code) = check_on("m396_match_binding_negative");
    assert_ne!(code, Some(0), "pattern-binding store must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'v'"),
        "expected the immutable-let message for 'v', got:\n{stderr}"
    );
}

/// m396: `&self` is immutable; only `&mut self` may store through the
/// receiver.
#[test]
fn m396_shared_ref_receiver_store_is_hard_error() {
    let (stderr, code) = check_on("m396_shared_ref_self_negative");
    assert_ne!(code, Some(0), "&self receiver store must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'self'"),
        "expected the immutable-let message for 'self', got:\n{stderr}"
    );
}

/// m396: the legal forms stay green (var rebind/field store, shadowing,
/// `var (a, b)`, params, for variables, `&mut self`).
#[test]
fn m396_mutable_bindings_accept() {
    let (stderr, code, exe) = run_on("m396_mutable_bindings_accept");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "legal mutable forms must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("cannot assign to immutable"),
        "no immutability diagnostic is allowed here, got:\n{stderr}"
    );
}

/// m396: `let x: T;` is init-once. The first whole-binding assignment is
/// the initialization and must compile/run; this is the stdlib deferred-init
/// shape (`let rc: Int32; ... rc = ...`).
#[test]
fn m396_deferred_init_accept() {
    let (stderr, code, exe) = run_on("m396_deferred_init_accept");
    let _ = std::fs::remove_file(&exe);
    assert_eq!(code, Some(0), "deferred init must compile. stderr:\n{stderr}");
    assert!(
        !stderr.contains("cannot assign to immutable"),
        "the first deferred store must not be rejected, got:\n{stderr}"
    );
}

/// m396: deferred init is ONCE. The second assignment must be rejected.
#[test]
fn m396_deferred_init_twice_is_hard_error() {
    let (stderr, code) = check_on("m396_deferred_init_twice_negative");
    assert_ne!(code, Some(0), "second store after deferred init must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'x'"),
        "expected the immutable-let message for 'x', got:\n{stderr}"
    );
}

/// m396: `let x: T = 0;` is initialized at declaration -- an explicit zero is
/// NOT a deferred-init placeholder.
#[test]
fn m396_initialized_zero_is_immutable() {
    let (stderr, code) = check_on("m396_initialized_zero_negative");
    assert_ne!(code, Some(0), "explicit `= 0` binding must be immutable. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'x'"),
        "expected the immutable-let message for 'x', got:\n{stderr}"
    );
}

/// m396: a field store is not an initialization of a deferred `let`; only the
/// first bare whole-binding store consumes the init-once state.
#[test]
fn m396_deferred_field_store_is_hard_error() {
    let (stderr, code) = check_on("m396_deferred_field_store_negative");
    assert_ne!(code, Some(0), "field store before deferred init must fail. stderr:\n{stderr}");
    assert!(
        stderr.contains("cannot assign to immutable binding 'p'"),
        "expected the immutable-let message for 'p', got:\n{stderr}"
    );
}

/// m396 regression lock: the real stdlib modules that carry deferred
/// `let x: T;` FFI/OS initializations (and `xiom.collections` receiver
/// fields) import cleanly -- zero T001.
#[test]
fn m396_stdlib_imports_have_no_immutable_errors() {
    let (stderr, code) = check_on("m396_stdlib_imports_accept");
    assert_eq!(code, Some(0), "stdlib import check must succeed. stderr:\n{stderr}");
    assert!(
        !stderr.contains("cannot assign to immutable"),
        "stdlib catalog bodies must have no immutable-binding errors, got:\n{stderr}"
    );
}

/// REL-20261010-1554-playground-2: `Int + Str` is an implicit conversion and
/// must be a hard checker error (SPEC 9: conversions are explicit). The old
/// behavior compiled `12 + 2.to_string()` and printed "122".
#[test]
fn rel_playground_int_str_addition_is_hard_error() {
    let (stderr, code) = check_on("rel_playground_int_str_reject");
    assert_ne!(
        code,
        Some(0),
        "Int + Str must fail the checker. stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("cannot add Str and Int"),
        "expected the mixed Str/Int error, got:\n{stderr}"
    );
    assert!(
        stderr.contains("convert.int_to_string"),
        "the diagnostic must point at the explicit conversion, got:\n{stderr}"
    );
}

/// REL-20261010-1554-playground-2 positive control: explicit
/// `convert.int_to_string` is accepted, compiles, and concatenates.
#[test]
fn rel_playground_explicit_int_to_string_accepts() {
    let (stderr, code, exe) = run_on("rel_playground_int_str_convert");
    let run = std::process::Command::new(&exe).output();
    let _ = std::fs::remove_file(&exe);
    assert_eq!(
        code,
        Some(0),
        "explicit convert.int_to_string must compile. stderr:\n{stderr}"
    );
    let run = run.expect("run explicit-conversion probe");
    assert_eq!(run.status.code(), Some(0), "explicit conversion probe must exit 0");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "122",
        "explicit conversions must concatenate to the expected Str"
    );
}

/// REL-20261010-1554-playground-3: `(2 + 2.5)` promotes to Float64 and
/// `.to_str()` must print "4.5" instead of falling through to the W005
/// `@to_str` stub (empty render / 0xC0000409).
#[test]
fn rel_playground_mixed_to_str_prints_float() {
    let (stderr, code, exe) = run_on("rel_playground_mixed_to_str");
    let run = std::process::Command::new(&exe).output();
    let _ = std::fs::remove_file(&exe);
    assert_eq!(
        code,
        Some(0),
        "mixed-expression .to_str() must compile. stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("W005"),
        "the mixed-expression .to_str() must not use the W005 stub, got:\n{stderr}"
    );
    let run = run.expect("run mixed-expression to_str probe");
    assert_eq!(run.status.code(), Some(0), "mixed-expression probe must exit 0");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "4.5",
        "mixed Int/Float .to_str() must print the Float64 text"
    );
}
