// Checker spec-gap locks (2026-11-03).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The `let` immutability gap showed the checker can accept programs the
// language spec rejects. This file pins the related gaps found by the
// 2026-11-03 audit. ALL locks below are active (0 ignored) since the
// enforcement change:
//
//   - assignment to an undeclared name
//   - struct literals missing a required field
//   - break/continue outside a loop (and unknown loop labels)
//   - duplicate function parameter names
//   - duplicate struct-literal field names (the spread base is not a field)
//   - duplicate pattern binding names, with the legal or-pattern rule:
//     alternatives may repeat names only when they bind the same set
//   - `const` initializers must be constant expressions (SPEC 2.6)
//   - fixed-width integer literals must fit (`let x: Int8 = 300` is rejected;
//     explicit `300 as Int8` remains the wrapping form)
//
// Positive controls at the bottom pin the still-legal shapes:
// mutable var/params/for bindings, legal or-patterns, constant-expression
// consts, in-range literals and explicit casts, and deferred `let` init-once.
//
// Not included because the current language semantics define them:
//   - `let x: T;` / `var x: T;` read before assignment yields 0 (zeroed slot),
//   - a non-unit function that falls through yields the zero value.

use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn check_source(name: &str, src: &str) -> (String, Option<i32>) {
    let path = std::env::temp_dir().join(format!(
        "xiom_specgap_{}_{}.xi",
        name,
        std::process::id()
    ));
    std::fs::write(&path, src).expect("write spec-gap source");
    let output = Command::new(xiom_bin())
        .arg("--check")
        .arg(&path)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let _ = std::fs::remove_file(&path);
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
    )
}

fn assert_rejected(name: &str, src: &str, needle: &str) {
    let (stderr, code) = check_source(name, src);
    assert_ne!(
        code,
        Some(0),
        "{name}: expected the checker to reject this program; stderr:\n{stderr}"
    );
    assert!(
        stderr.to_lowercase().contains(needle),
        "{name}: expected stderr to mention '{needle}'; stderr:\n{stderr}"
    );
}

fn assert_accepted(name: &str, src: &str) {
    let (stderr, code) = check_source(name, src);
    assert_eq!(
        code,
        Some(0),
        "{name}: expected the checker to accept this program; stderr:\n{stderr}"
    );
}

// ---- enforcement that already holds (baseline for the harness) -------------

#[test]
fn undefined_variable_is_rejected() {
    assert_rejected(
        "undefined_variable",
        "fn main() { z = 1; }\n",
        "undefined variable",
    );
}

#[test]
fn struct_literal_missing_field_is_rejected() {
    assert_rejected(
        "missing_struct_field",
        "type D = { a: Int; b: Int; }\nfn main() { let d = D { a: 1; }; }\n",
        "missing field",
    );
}

// ---- enforced locks --------------------------------------------------------

#[test]
fn break_outside_loop_is_rejected() {
    assert_rejected("break_outside", "fn main() { break; }\n", "break outside");
}

#[test]
fn continue_outside_loop_is_rejected() {
    assert_rejected(
        "continue_outside",
        "fn main() { continue; }\n",
        "continue outside",
    );
}

#[test]
fn unknown_loop_label_is_rejected() {
    assert_rejected(
        "unknown_label",
        "fn main() { while true { break @missing; } }\n",
        "unknown loop label",
    );
}

#[test]
fn duplicate_parameter_name_is_rejected() {
    assert_rejected(
        "duplicate_param",
        "fn f(x: Int, x: Int) -> Int { return x; }\nfn main() { }\n",
        "duplicate parameter",
    );
}

#[test]
fn duplicate_receiver_name_is_rejected() {
    assert_rejected(
        "duplicate_receiver",
        "type T = { v: Int; }\nfn T.m(self, self) -> Int { return self.v; }\nfn main() { }\n",
        "duplicate parameter",
    );
}

#[test]
fn duplicate_struct_literal_field_is_rejected() {
    assert_rejected(
        "duplicate_field",
        "type D = { v: Int; }\nfn main() { let d = D { v: 1; v: 2; }; }\n",
        "duplicate field",
    );
}

#[test]
fn duplicate_pattern_binding_is_rejected() {
    assert_rejected(
        "duplicate_pattern",
        "fn main() { let t = (1, 2); match t { (x, x) => { } }; }\n",
        "duplicate binding",
    );
}

#[test]
fn or_pattern_with_different_bindings_is_rejected() {
    assert_rejected(
        "or_pattern_sets",
        "enum E { A(Int), B }\nfn f(e: E) -> Int { match e { A(x) | B => { return x; } }; return 0; }\nfn main() { }\n",
        "same names",
    );
}

#[test]
fn non_constant_const_initializer_is_rejected() {
    // CTFE folds user-function calls, so the negative lock must use a
    // genuinely non-CTFE construct: an extern/FFI call.
    assert_rejected(
        "const_non_const",
        "extern \"C\" { fn ffi_value() -> Int; }\nconst C: Int = unsafe { ffi_value() };\nfn main() { }\n",
        "not a constant expression",
    );
}

#[test]
fn non_constant_const_identifier_is_rejected() {
    assert_rejected(
        "const_non_const_ident",
        "var g: Int = 1;\nconst C: Int = g;\nfn main() { }\n",
        "not a constant expression",
    );
}

#[test]
fn out_of_range_int_literal_is_rejected() {
    assert_rejected(
        "int_literal_range",
        "fn main() { let x: Int8 = 300; }\n",
        "out of range",
    );
}

#[test]
fn out_of_range_negative_int_literal_is_rejected() {
    assert_rejected(
        "int_literal_range_neg",
        "fn main() { let x: Int8 = -129; }\n",
        "out of range",
    );
}

#[test]
fn out_of_range_unsigned_literal_is_rejected() {
    assert_rejected(
        "uint_literal_range",
        "fn main() { let x: UInt8 = 256; }\n",
        "out of range",
    );
}

#[test]
fn out_of_range_call_argument_is_rejected() {
    assert_rejected(
        "arg_literal_range",
        "fn f(x: Int8) -> Int { return x; }\nfn main() { f(300); }\n",
        "out of range",
    );
}

#[test]
fn out_of_range_struct_field_is_rejected() {
    assert_rejected(
        "field_literal_range",
        "type D = { v: Int8; }\nfn main() { let d = D { v: 300; }; }\n",
        "out of range",
    );
}

#[test]
fn out_of_range_return_is_rejected() {
    assert_rejected(
        "return_literal_range",
        "fn f() -> Int8 { return 300; }\nfn main() { }\n",
        "out of range",
    );
}

#[test]
fn out_of_range_const_initializer_is_rejected() {
    assert_rejected(
        "const_literal_range",
        "const C: Int8 = 300;\nfn main() { }\n",
        "out of range",
    );
}

// ---- positive controls -----------------------------------------------------

#[test]
fn var_params_and_for_bindings_stay_mutable() {
    assert_accepted(
        "mutable_bindings",
        "fn bump(x: Int, y: Int) -> Int {\n    x = x + y;\n    return x;\n}\n\
         var g: Int = bump(1, 2);\n\
         fn main() {\n    var a = 1;\n    a = 2;\n    for i in [1, 2] {\n        a = a + i;\n    }\n}\n",
    );
}

#[test]
fn loop_control_inside_loops_is_accepted() {
    assert_accepted(
        "loop_control_ok",
        "fn main() {\n    while true { break; }\n    for i in [1, 2] { continue; }\n    @outer: while true { break @outer; }\n}\n",
    );
}

#[test]
fn legal_or_patterns_are_accepted() {
    assert_accepted(
        "or_pattern_ok",
        "enum E { A(Int), B(Int) }\n\
         fn pick(e: E) -> Int {\n    match e { A(x) | B(x) => { return x; } };\n    return 0;\n}\n\
         fn main() { }\n",
    );
}

#[test]
fn constant_expression_consts_are_accepted() {
    assert_accepted(
        "const_ok",
        "const A: Int = 1 + 2 * 3;\nconst B: Int = -A;\nconst C: Str = \"ok\";\n\
         const D: Bool = B < 100;\nconst E: (Int, Int) = (1, 2);\nfn main() { }\n",
    );
}

#[test]
fn ctfe_call_if_match_and_builtin_consts_are_accepted() {
    // XIOM's CTFE engine folds pure user-function calls, if/match over const
    // values, and the sizeof/align_of/type_id/is_signed/field_offset builtins.
    assert_accepted(
        "const_ctfe_ok",
        "fn factorial(n: Int) -> Int {\n    if n <= 1 { return 1; }\n    return n * factorial(n - 1);\n}\n\
         const CALL: Int = factorial(5);\n\
         const IF_C: Int = if true { 42 } else { 0 };\n\
         const MATCH_C: Int = match Some(7) { Some(v) => v, None => 0 };\n\
         const BUILTIN: Int = sizeof::<Int>();\n\
         fn main() { }\n",
    );
}

#[test]
fn in_range_literals_and_explicit_casts_are_accepted() {
    assert_accepted(
        "range_ok",
        "fn main() {\n    let a: Int8 = 127;\n    let b: Int8 = -128;\n    let c: UInt8 = 255;\n\
         let d: UInt8 = 0;\n    let e: Int8 = 300 as Int8;\n    let f: UInt8 = (250) as UInt8;\n}\n",
    );
}

#[test]
fn deferred_let_init_once_stays_legal() {
    assert_accepted(
        "deferred_init_ok",
        "fn main() { let x: Int; x = 5; }\n",
    );
}
