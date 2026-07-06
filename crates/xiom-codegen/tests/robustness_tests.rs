// XIOM — Compiler Robustness Tests
// Verifies the compiler never panics — always returns Result, never crashes.

use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_codegen::IrEmitter;

/// Attempt full compile. Returns Ok(ir) or Err(msg). MUST NOT panic.
fn try_compile(source: &str) -> Result<String, String> {
    let tokens = Lexer::new(source).tokenize();
    let program = Parser::new(tokens).parse_program().map_err(|e| e.to_string())?;
    let mut emitter = IrEmitter::new();
    emitter.compile_program(&program)
}

/// Assert compilation does not panic (result can be Ok or Err).
fn assert_no_panic(source: &str) {
    let _ = try_compile(source); // must not panic
}

// =====================================================================
// Category 1: Match edge cases (the crash we just fixed)
// =====================================================================

#[test]
fn robust_match_many_mixed_arms() {
    assert_no_panic("enum E { A, B(x: Int), C, D(y: Int), F, G(z: Int) } fn f(e: E) -> Int { match e { A => 1, B(x) => x, C => 3, D(y) => y, F => 5, G(z) => z } } fn main() -> Int { return 0; }");
}

#[test]
fn robust_match_all_wildcards() {
    assert_no_panic("fn f(x: Int) -> Int { match x { _ => 0 } } fn main() -> Int { return 0; }");
}

#[test]
fn robust_match_empty_arms() {
    assert_no_panic("enum E { A } fn f(e: E) -> Int { match e { A => {} } return 0; } fn main() -> Int { return 0; }");
}

#[test]
fn robust_match_nested() {
    assert_no_panic("fn f(x: Option[Option[Int]]) -> Int { match x { Some(Some(v)) => v, Some(None) => 0, None => -1 } } fn main() -> Int { return 0; }");
}

#[test]
fn robust_match_or_patterns() {
    assert_no_panic("fn f(c: Char) -> Int { match c { 'a' | 'b' | 'c' => 1, _ => 0 } } fn main() -> Int { return 0; }");
}

// =====================================================================
// Category 2: Malformed input (must error, not panic)
// =====================================================================

#[test]
fn robust_unclosed_brace() { assert_no_panic("fn main() -> Int { return 0;"); }

#[test]
fn robust_unclosed_paren() { assert_no_panic("fn main() -> Int { return foo(; }"); }

#[test]
fn robust_empty_input() { assert_no_panic(""); }

#[test]
fn robust_only_whitespace() { assert_no_panic("   \n\t  "); }

#[test]
fn robust_incomplete_fn() { assert_no_panic("fn"); }

#[test]
fn robust_incomplete_match() { assert_no_panic("fn f() -> Int { match"); }

#[test]
fn robust_deeply_nested_parens() {
    let src = format!("fn main() -> Int {{ return {}1{}; }}", "(".repeat(10), ")".repeat(10));
    assert_no_panic(&src);
}

#[test]
fn robust_unknown_type() { assert_no_panic("fn f(x: NonexistentType) -> Int { return 0; }"); }

#[test]
fn robust_empty_match() { assert_no_panic("fn f(x: Int) -> Int { match x { } } fn main() -> Int { return 0; }"); }

// =====================================================================
// Category 3: Type edge cases
// =====================================================================

#[test]
fn robust_deeply_nested_generics() { assert_no_panic("fn main() -> Int { var x: Vec[Vec[Vec[Vec[Int]]]] = Vec[Vec[Vec[Vec[Int]]]].new(); return 0; }"); }

#[test]
fn robust_recursive_type() { assert_no_panic("type Node = { next: Node; val: Int; } fn main() -> Int { return 0; }"); }

#[test]
fn robust_many_fields() {
    let fields: String = (0..100).map(|i| format!("f{}: Int; ", i)).collect();
    assert_no_panic(&format!("type Big = {{ {} }} fn main() -> Int {{ return 0; }}", fields));
}

#[test]
fn robust_empty_struct() { assert_no_panic("type Empty = { } fn main() -> Int { return 0; }"); }

#[test]
fn robust_int_char_casts() { assert_no_panic("fn main() -> Int { var c = 65 as Char; var i = c as Int; return i; }"); }

// =====================================================================
// Category 4: Expression edge cases
// =====================================================================

#[test]
fn robust_deep_arithmetic() {
    let expr = (0..10).map(|_| "1 + ").collect::<String>() + "1";
    assert_no_panic(&format!("fn main() -> Int {{ return {}; }}", expr));
}

#[test]
fn robust_deep_method_chain() {
    let chain = ".clone()".repeat(10);
    assert_no_panic(&format!("fn main() -> Int {{ var x = 5; return x{}; }}", chain));
}

#[test]
fn robust_bitwise_chain() { assert_no_panic("fn main() -> Int { return 1 & 2 | 3 ^ 4; }"); }

#[test]
fn robust_match_expression() { assert_no_panic("fn main() -> Int { var x = match 1 { 1 => 10, _ => 0 }; return x; }"); }

#[test]
fn robust_unsafe_block() { assert_no_panic("fn main() -> Int { unsafe { var x = 1; } return 0; }"); }

#[test]
fn robust_break_continue_outside_loop() {
    // break/continue outside a loop — must not panic (may error)
    assert_no_panic("fn main() -> Int { break; return 0; }");
    assert_no_panic("fn main() -> Int { continue; return 0; }");
}

#[test]
fn robust_break_continue_in_loop() {
    assert_no_panic("fn main() -> Int { var i = 0; while i < 10 { if i == 5 { break; } i = i + 1; } return i; }");
}

// =====================================================================
// Category 5: Div-by-zero and overflow guards (should compile with traps)
// =====================================================================

#[test]
fn robust_div_by_literal_zero() { assert_no_panic("fn main() -> Int { return 1 / 0; }"); }

#[test]
fn robust_mod_by_zero() { assert_no_panic("fn main() -> Int { return 1 % 0; }"); }

#[test]
fn robust_huge_int_literal() { assert_no_panic("fn main() -> Int { return 999999999999999999; }"); }
