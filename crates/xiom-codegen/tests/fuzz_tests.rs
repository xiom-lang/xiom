// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM — Compiler Fuzz / Robustness Tests (GAP 5)
// Verifies the compiler NEVER panics on adversarial input — it must always
// return a Result (Ok or Err), never crash the process.
//
// Mirrors the in-process pipeline used by `robustness_tests.rs`:
//     Lexer::new(src).tokenize() -> Parser::new(tokens).parse_program()
//     -> IrEmitter::new().compile_program(&program)
//
// Every test funnels through `compile_no_panic`, which wraps the pipeline in
// `std::panic::catch_unwind` and asserts no panic occurred, regardless of the
// Ok/Err outcome. Inputs that must be *rejected* (e.g. nesting past the
// parser's depth guard, MAX_EXPR_DEPTH = 32) additionally assert a clean Err.
//
// Run with:
//     cargo test -p xiom-codegen --test fuzz_tests

use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_codegen::IrEmitter;

/// Full in-process compile pipeline. Returns Ok(ir) or Err(msg). MUST NOT panic.
fn compile(source: &str) -> Result<String, String> {
    // Run on a dedicated big-stack thread: `compile_expr` has very large
    // debug-build frames and Rust test threads default to a 2MB stack —
    // nested expressions overflowed at trivial depth (5c.30).
    let src = source.to_string();
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let tokens = Lexer::new(&src).tokenize();
            let program = Parser::new(tokens).parse_program().map_err(|e| e.to_string())?;
            let mut emitter = IrEmitter::new();
            emitter.compile_program(&program)
        })
        .expect("spawn compile thread")
        .join()
        .expect("compile thread must not panic")
}

/// Run the pipeline inside `catch_unwind` and assert it did NOT panic.
/// Returns the inner Result so callers can additionally inspect Ok/Err.
fn compile_no_panic(source: &str) -> Result<String, String> {
    let outcome = std::panic::catch_unwind(|| compile(source));
    assert!(
        outcome.is_ok(),
        "compiler PANICKED on adversarial input (source length = {} bytes)",
        source.len()
    );
    outcome.unwrap()
}

/// Assert the pipeline returns (Ok or Err) without panicking.
fn assert_no_panic(source: &str) {
    let _ = compile_no_panic(source);
}

/// Assert the pipeline returns a clean Err (rejection) — no panic, no Ok.
/// Used for inputs that must be rejected, e.g. nesting past the depth guard.
/// NOTE: With Phase 5c error recovery, the guard fires but recovery may
/// salvage a partial program. Use `assert_parser_error` for guard tests.

/// Assert the parser records at least one error (e.g. depth guard fired).
/// Error recovery may still produce a valid partial program (5c recovery),
/// but the guard must have triggered.
fn assert_parser_error(source: &str) {
    let src = source.to_string();
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let tokens = Lexer::new(&src).tokenize();
            let mut parser = Parser::new(tokens);
            let _ = parser.parse_program();
            assert!(
                !parser.errors().is_empty(),
                "expected parser errors (e.g. depth guard), but parser recorded none"
            );
        })
        .expect("spawn parser thread")
        .join()
        .expect("parser thread must not panic");
}

// =====================================================================
// Category 1: Random / garbage token soup
// =====================================================================

#[test]
fn fuzz_garbage_token_soup_symbols() {
    assert_no_panic("@@@ ### $$$ %%% ^^^ &&& *** ((( ))) !!! ~~~ ??? ::: ;;; ,,, ... ..= => -> <- |> <|");
}

#[test]
fn fuzz_garbage_token_soup_mixed() {
    assert_no_panic("fn 123 !@# { } ( ) [ ] => -> :: ; , . .. ... 3.14.15.9 0xZZ 0b222 'unterminated 999e999");
}

#[test]
fn fuzz_keyword_soup() {
    assert_no_panic("pub fn async unsafe extern spawn await match enum type interface impl let var const return if else while for break continue requires ensures");
}

#[test]
fn fuzz_random_control_and_symbol_soup() {
    // Valid UTF-8 but semantically meaningless — the lexer/parser must reject
    // gracefully, not crash.
    assert_no_panic("你好 ☃ 🎉 \u{7} \u{1b}[0m `backtick` \\ / | ~ ^ % $ # @ ! ? : ; = + - * << >> && || == != <= >=");
}

// =====================================================================
// Category 2: Deep nesting around the parser depth guard (MAX_EXPR_DEPTH = 32)
// =====================================================================

#[test]
fn fuzz_nested_array_brackets_under_guard() {
    // Bracket-soup array-type nesting `[[[[...]]]]` (~25 deep). Not valid
    // XIOM array syntax (`[N]T`), so this must reject cleanly, never panic.
    let src = format!(
        "fn f(x: {}Int{}) -> Int {{ return 0; }} fn main() -> Int {{ return 0; }}",
        "[".repeat(25),
        "]".repeat(25)
    );
    assert_no_panic(&src);
}

#[test]
fn fuzz_valid_nested_arrays_under_guard() {
    // Well-formed nested fixed-array type `[1][1]...Int` (~25 deep), safely
    // under the depth guard — must parse without panicking.
    let src = format!(
        "fn f(x: {}Int) -> Int {{ return 0; }} fn main() -> Int {{ return 0; }}",
        "[1]".repeat(25)
    );
    assert_no_panic(&src);
}

#[test]
fn fuzz_nested_generics_under_guard() {
    // `Vec[Vec[...Int...]]` ~25 levels deep — under the guard, should parse.
    let src = format!(
        "fn f(x: {}Int{}) -> Int {{ return 0; }} fn main() -> Int {{ return 0; }}",
        "Vec[".repeat(25),
        "]".repeat(25)
    );
    assert_no_panic(&src);
}

#[test]
fn fuzz_nested_generics_over_guard() {
    // `Vec[Vec[...]]` 300 levels deep -- over MAX_EXPR_DEPTH (128; this test
    // was written when the guard was 32 and went stale). The parser must
    // record an error (guard triggered), even if 5c error recovery salvages
    // a partial program from remaining declarations.
    let src = format!(
        "fn f(x: {}Int{}) -> Int {{ return 0; }} fn main() -> Int {{ return 0; }}",
        "Vec[".repeat(300),
        "]".repeat(300)
    );
    assert_parser_error(&src);
}

#[test]
fn fuzz_deeply_nested_parens_over_guard() {
    // 300 nested parens -- well over MAX_EXPR_DEPTH (128). Parser must record
    // an error, even if 5c recovery salvages the rest.
    let src = format!(
        "fn main() -> Int {{ return {}1{}; }}",
        "(".repeat(300),
        ")".repeat(300)
    );
    assert_parser_error(&src);
}

// =====================================================================
// Category 3: Large-but-structured programs
// =====================================================================

#[test]
fn fuzz_huge_match_many_arms() {
    // A match with 500 integer-literal arms plus a wildcard.
    let arms: String = (0..500).map(|i| format!("{} => {}, ", i, i)).collect();
    let src = format!(
        "fn main() -> Int {{ var x = 0; return match x {{ {}_ => -1 }}; }}",
        arms
    );
    assert_no_panic(&src);
}

#[test]
fn fuzz_large_valid_arithmetic_expr() {
    // Generated valid left-associative arithmetic expression. Binary operators
    // parse in a loop (not recursively) so the PARSER stays under its depth
    // guard, but the resulting left-leaning BinOp tree is walked recursively by
    // the checker AND codegen (the latter also runs `infer_llvm_type` per level),
    // so even moderately deep chains can exhaust the 2 MB test-thread stack.
    // Kept small (12) so the codegen recursion is comfortably safe. See
    // `fuzz_deep_arithmetic_overflows_known_limitation` below for the documented
    // deep-recursion limit (the durable fix is an explicit work-stack in codegen).
    let expr = (0..12).map(|_| "1 + ").collect::<String>() + "1";
    let src = format!("fn main() -> Int {{ return {}; }}", expr);
    assert_no_panic(&src);
}

// KNOWN LIMITATION (pre-existing, not from stdlib work): a very deep but VALID
// left-associative expression builds a deep AST that the checker/codegen walk
// recursively, overflowing the native stack. Fixed by iterative BinOp flattening
// in compile_expr — deep same-operator chains are compiled via an explicit work
// list instead of recursive descent. The test now exercises the hardened compiler.
#[test]
fn fuzz_deep_arithmetic_overflows_known_limitation() {
    let expr = (0..5000).map(|_| "1 + ").collect::<String>() + "1";
    let src = format!("fn main() -> Int {{ return {}; }}", expr);
    let _ = compile(&src); // may overflow the stack — documents the limitation
}

#[test]
fn fuzz_extremely_long_identifier() {
    // A single ~100k-char identifier — must not panic (may parse or error).
    let name = "a".repeat(100_000);
    let src = format!("fn main() -> Int {{ var {} = 1; return {}; }}", name, name);
    assert_no_panic(&src);
}

// =====================================================================
// Category 4: Malformed contract clauses
// =====================================================================

#[test]
fn fuzz_malformed_requires_no_expr() {
    // `requires:` with no following expression (body brace immediately).
    assert_no_panic("fn f(a: Int) -> Int requires: { return a; } fn main() -> Int { return 0; }");
}

#[test]
fn fuzz_malformed_requires_dangling() {
    // `requires:` followed by a dangling operator, then EOF.
    assert_no_panic("fn f(a: Int) -> Int requires: a >");
}

#[test]
fn fuzz_malformed_ensures_unterminated() {
    // `ensures:` with an unterminated comparison expression.
    assert_no_panic("fn f(a: Int) -> Int ensures: result == { return a; }");
}

#[test]
fn fuzz_malformed_ensures_eof() {
    // `ensures:` then abrupt end of input.
    assert_no_panic("fn f(a: Int) -> Int ensures:");
}

// =====================================================================
// Category 5: Unbalanced brackets of each kind
// =====================================================================

#[test]
fn fuzz_unbalanced_parens() {
    assert_no_panic("fn main() -> Int { return (((((1; }");
}

#[test]
fn fuzz_unbalanced_braces() {
    assert_no_panic("fn main() -> Int {{{ return 0; }");
}

#[test]
fn fuzz_unbalanced_brackets() {
    assert_no_panic("fn main() -> Int { var x = [[[[1; return 0; }");
}

#[test]
fn fuzz_unbalanced_angle_brackets() {
    assert_no_panic("fn f(x: Vec<Vec<Vec<Int>) -> Int { return 0; } fn main() -> Int { return 0; }");
}

// =====================================================================
// Category 6: Mixed unicode / escape sequences in string literals
// =====================================================================

#[test]
fn fuzz_unicode_in_strings() {
    assert_no_panic(r#"fn main() -> Int { var s = "héllo wörld ☃ 你好 🎉 Ω≈ç√∫"; return 0; }"#);
}

#[test]
fn fuzz_escape_sequences_in_strings() {
    assert_no_panic(r#"fn main() -> Int { var s = "a\tb\nc\\d\"e\r\0f"; return 0; }"#);
}

#[test]
fn fuzz_mixed_unicode_escape_string() {
    assert_no_panic(r#"fn main() -> Int { var s = "\u{1F600}\t\x41 café ☃\n\\ end"; return 0; }"#);
}
