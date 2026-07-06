// XIOM — Feature Regression Tests
// Locks in every syntax/semantic feature added during stdlib hardening.

use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_codegen::IrEmitter;

fn compile(source: &str) -> Result<String, String> {
    let tokens = Lexer::new(source).tokenize();
    let program = Parser::new(tokens).parse_program().map_err(|e| e.to_string())?;
    let mut emitter = IrEmitter::new();
    emitter.compile_program(&program)
}

// =====================================================================
// Lexer features
// =====================================================================

#[test]
fn regress_hex_literals() {
    let ir = compile("fn main() -> Int { return 0xFF; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_hex_uppercase() {
    let ir = compile("fn main() -> Int { return 0xDEADBEEF; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_scientific_notation() {
    let ir = compile("fn main() -> Float64 { return 1.5e10; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_scientific_negative_exp() {
    let ir = compile("fn main() -> Float64 { return 2.5e-8; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_caret_operator() {
    let ir = compile("fn f(a: Int, b: Int) -> Int { return a ^ b; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("xor"));
}

#[test]
fn regress_tilde_operator() {
    let ir = compile("fn f(a: Int) -> Int { return ~a; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

// =====================================================================
// Bitwise operators
// =====================================================================

#[test]
fn regress_bitwise_and() {
    let ir = compile("fn f(a: Int, b: Int) -> Int { return a & b; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("and"));
}

#[test]
fn regress_bitwise_or() {
    let ir = compile("fn f(a: Int, b: Int) -> Int { return a | b; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains(" or "));
}

#[test]
fn regress_shift_left() {
    let ir = compile("fn f(a: Int) -> Int { return a << 2; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("shl"));
}

#[test]
fn regress_shift_right() {
    let ir = compile("fn f(a: Int) -> Int { return a >> 2; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("ashr"));
}

// =====================================================================
// Parser features
// =====================================================================

#[test]
fn regress_extern_block() {
    let ir = compile("extern \"C\" { fn malloc(size: Int) -> *UInt8; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("declare"));
}

#[test]
fn regress_extern_variadic() {
    let ir = compile("extern \"C\" { fn printf(fmt: *UInt8, ...) -> Int32; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("declare"));
}

#[test]
fn regress_pointer_types() {
    let ir = compile("fn f(p: *UInt8) -> Int { return 0; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_const_pointer() {
    let ir = compile("fn f(p: *const UInt8) -> Int { return 0; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_mut_pointer() {
    let ir = compile("fn f(p: *mut UInt8) -> Int { return 0; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_angle_bracket_generics() {
    let ir = compile("fn f() -> Result<Int, Str> { return Ok(1); } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_generic_bounds() {
    let ir = compile("interface Ord { fn compare(o: &Self) -> Int; } fn max[T: Ord](a: T, b: T) -> T { return a; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_const_generic() {
    let ir = compile("fn len[T, const N: Int](arr: [N]T) -> Int { return 0; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_dotted_type() {
    let ir = compile("module m\ntype Err = { code: Int; } fn f(e: m.Err) -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_top_level_var() {
    let ir = compile("var counter: Int = 0; fn main() -> Int { return counter; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_pub_const() {
    let ir = compile("pub const MAX: Int = 100; fn main() -> Int { return MAX; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_uninitialized_var() {
    let ir = compile("fn main() -> Int { var x: Int; x = 5; return x; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_local_const() {
    let ir = compile("fn f() -> Int { const K: Int = 42; return K; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_struct_literal_semicolons() {
    let ir = compile("type P = { x: Int; y: Int; } fn main() -> Int { var p = P{ x: 1; y: 2; }; return p.x; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_struct_spread() {
    let ir = compile("type P = { x: Int; y: Int; } fn main() -> Int { var base = P{ x: 0, y: 0 }; var p = P{ x: 1, ..base }; return p.x; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_inline_enum_alias() {
    let ir = compile("type Color = enum { Red, Green, Blue } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_position_only_enum_variant() {
    let ir = compile("enum Token { Ident(Str), Num(Int) } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_scope_resolution() {
    let ir = compile("type Vec2 = { x: Int; } fn Vec2.new() -> Vec2 { return Vec2{ x: 0 }; } fn main() -> Int { var v = Vec2::new(); return v.x; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_or_patterns() {
    let ir = compile("fn f(x: Int) -> Int { match x { 1 | 2 | 3 => 1, _ => 0 } } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_match_expression() {
    let ir = compile("fn main() -> Int { var x = match 5 { 5 => 50, _ => 0 }; return x; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_unit_literal() {
    let ir = compile("fn f() -> Result[Int, Str] { return Ok(1); } fn main() -> Int { match f() { Ok(v) => return v, Err(e) => return 0 } }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_return_without_semicolon() {
    let ir = compile("fn f() -> Int { return 5 } fn main() -> Int { return f(); }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_dotted_enum_pattern() {
    let ir = compile("enum Color { Red, Green } fn f(c: Color) -> Int { match c { Color.Red => 1, Color.Green => 2 } } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

// =====================================================================
// break/continue
// =====================================================================

#[test]
fn regress_break() {
    let ir = compile("fn main() -> Int { var i = 0; while i < 10 { if i == 5 { break; } i = i + 1; } return i; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_continue() {
    let ir = compile("fn main() -> Int { var i = 0; var s = 0; while i < 10 { i = i + 1; if i == 5 { continue; } s = s + 1; } return s; }").unwrap();
    assert!(ir.contains("define"));
}

// =====================================================================
// Casts
// =====================================================================

#[test]
fn regress_int_to_char_cast() {
    let ir = compile("fn main() -> Int { var c = 65 as Char; return c as Int; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_int_to_uint_cast() {
    let ir = compile("fn main() -> UInt { return 42 as UInt; }").unwrap();
    assert!(ir.contains("define"));
}

// =====================================================================
// Primitive Ord/Eq methods
// =====================================================================

#[test]
fn regress_primitive_compare() {
    let ir = compile("fn f(a: Int, b: Int) -> Int { return a.compare(b); } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_primitive_eq() {
    let ir = compile("fn f(a: Int, b: Int) -> Bool { return a.eq(b); } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}
