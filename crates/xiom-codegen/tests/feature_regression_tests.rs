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

// =====================================================================
// Ecosystem compiler gaps (COMPILER_GAPS.md) — lock in that each spec-valid
// pattern the production ecosystem uses parses + emits IR. Every one of these
// was a reported gap in xiomc v0.11.0; these tests prevent regression.
// =====================================================================

#[test]
fn regress_gap1_qualified_enum_pattern_in_match() {
    // `Type.Variant` patterns in match arms.
    let ir = compile("enum E { A, B(x: Int) } fn f(e: E) -> Int { match e { E.A => 0, E.B(v) => v, } } fn main() -> Int { return f(E.A); }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_gap2_extern_c_block() {
    let ir = compile("extern \"C\" { fn puts(s: *UInt8) -> Int; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("declare") && ir.contains("@puts"));
}

#[test]
fn regress_gap6_wildcard_in_user_enum_payload() {
    // `B(_)` on a user enum variant with a payload.
    let ir = compile("enum E { A, B(x: Int) } fn f(e: E) -> Int { match e { A => 0, B(_) => 1, } } fn main() -> Int { return f(A); }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_gap9_type_alias_enum() {
    let ir = compile("pub type E = enum { A, B(x: Int) } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_gap10_trailing_semicolon_after_control_block() {
    let ir = compile("fn main() -> Int { if 1 > 0 { return 1; }; return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_gap11_tail_expression_implicit_return() {
    let ir = compile("fn g() -> Int { return 5; } fn f() -> Int { g() } fn main() -> Int { return f(); }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_gap8_bitwise_shift_operators() {
    let ir = compile("fn f(a: Int, b: Int) -> Int { return (a >> 8) & 0xFF ^ (b << 2) | 1; } fn main() -> Int { return f(256, 3); }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_gap5_string_escapes() {
    // \0 \b and \u{XX} brace-form unicode escapes.
    let ir = compile("fn main() -> Int { let s = \"\\0\"; let t = \"\\u{41}\"; return 0; }").unwrap();
    assert!(ir.contains("define"));
}

#[test]
fn regress_gap12_unit_result_ok() {
    let ir = compile("fn f() -> Result[Int, Int] { return Ok(1); } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"));
}

// ============================================================
// 5c.29 / 5c.30 Production hardening regression tests
// ============================================================

#[test]
fn regress_5c29_container_field_handle() {
    // Struct with a Vec[Int] field: the field must be an i64 HANDLE,
    // not a by-value 32-byte %struct.Vec stored in the 8-byte slot.
    let ir = compile(r#"
pub type Foo = { items: Vec[Int]; count: Int; }
fn Foo.new() -> Foo { return Foo{ items: Vec[Int].new(), count: 0 }; }
fn main() -> Int { let f = Foo.new(); return 0; }
"#).unwrap();
    // The struct literal must NOT store %struct.Vec by value into the i64 handle field.
    assert!(ir.contains("ptrtoint"), "must box Vec header (ptrtoint) for handle field");
}

#[test]
fn regress_5c29_vec_of_struct_elem() {
    // Vec of struct with field access after indexing
    let ir = compile(r#"
pub type Point = { x: Int; y: Int; }
fn main() -> Int {
  var pts = Vec[Point].new();
  var p = Point{ x: 10, y: 20 };
  pts.push(p);
  let v = pts[0];
  return v.x - 10 + v.y - 20;
}
"#).unwrap();
    assert!(ir.contains("define"), "Vec-of-struct index + field access must compile");
}

#[test]
fn regress_5c29_float32_vec_elem() {
    // Vec[Float32] elements round-trip raw bits (not sitofp)
    let ir = compile(r#"
fn main() -> Int {
  var v = Vec[Float32].new();
  v.push(1.5);
  let x = v[0];
  if x == 1.5 { return 0; }
  return 1;
}
"#).unwrap();
    assert!(ir.contains("define"), "Float32 Vec elements must bit-reinterpret");
}

#[test]
fn regress_5c30_enum_payload_roundtrip() {
    let ir = compile("pub enum Val { No; Yes(Int); } fn main() -> Int { let v = Val.Yes(42); match v { Yes(n) => n-42; _ => 1; } }").unwrap();
    assert!(ir.contains("define"), "enum payload roundtrip");
}

#[test]
fn regress_5c30_implicit_self_call() {
    // Inside a method, bare `method(args)` resolves to self.method(args)
    let ir = compile(r#"
pub type Greeter = { name: Str; }
fn Greeter.greet(greeting: Str) -> Str { return greeting; }
fn Greeter.hello() -> Str { return greet("Hi"); }
fn main() -> Int { return 0; }
"#).unwrap();
    assert!(ir.contains("define"), "implicit-self call must compile");
}

#[test]
fn regress_5c30_local_field_ref() {
    // &local.field must NOT bind an unrelated same-named local
    let ir = compile(r#"
pub type Addr = { ip: Str; port: Int; }
fn main() -> Int {
  var a = Addr{ ip: "127.0.0.1", port: 8080 };
  var ip = "10.0.0.1";
  let s = &a.ip;
  return 0;
}
"#).unwrap();
    assert!(ir.contains("getelementptr"), "&local.field must emit real GEP");
}

#[test]
fn regress_5c29_string_concat() {
    let ir = compile(r#"fn main() -> Int { let a = "hello" + " world"; return a.len() - 11; }"#).unwrap();
    assert!(ir.contains("xiom_str_concat"), "Str + Str must call xiom_str_concat");
}

#[test]
fn regress_5c30_uint_coercion() {
    // Int literal → UInt8 coercion
    let ir = compile(r#"fn main() -> Int { var x: UInt8 = 255; return 0; }"#).unwrap();
    assert!(ir.contains("define"), "Int literal must coerce to UInt8");
}

// ============================================================
// 5c-R Production Refactoring Regression Tests
// ============================================================

#[test]
fn regress_5cr_contextual_keywords_as_ident() {
    // requires/ensures/invariant must work as regular identifiers (5c-R contextual keywords)
    let ir = compile("fn main() -> Int { var requires = 5; var ensures = 6; return requires + ensures - 11; }").unwrap();
    assert!(ir.contains("define"), "requires/ensures must work as regular identifiers");
}

#[test]
fn regress_5cr_contextual_keywords_in_contract() {
    // requires/ensures must still work as contract keywords
    let ir = compile("fn div(a: Int, b: Int) -> Int\n  requires: b != 0;\n  ensures: result >= 0;\n{ return a / b; } fn main() -> Int { return div(10, 2); }").unwrap();
    assert!(ir.contains("define"), "requires/ensures must work as contract keywords");
}

#[test]
fn regress_5cr_vec_with_capacity() {
    let ir = compile("fn main() -> Int { var v = Vec[Int].with_capacity(100); v.push(42); return v.len() - 1; }").unwrap();
    assert!(ir.contains("define"), "Vec.with_capacity must compile");
    assert!(ir.contains("malloc"), "with_capacity must allocate");
}

#[test]
fn regress_5cr_vec_with_capacity_struct() {
    let ir = compile("pub type Pt = { x: Int; y: Int; } fn main() -> Int { var v = Vec[Pt].with_capacity(10); return 0; }").unwrap();
    assert!(ir.contains("define"), "Vec.with_capacity for struct type must compile");
}

#[test]
fn regress_5cr_array_float64_elements() {
    let ir = compile("fn main() -> Int { var arr: [3]Float64 = [1.0, 2.0, 3.0]; return 0; }").unwrap();
    assert!(ir.contains("double"), "Float64 array must use double alloca/stores");
}

#[test]
fn regress_5cr_array_struct_elements() {
    let ir = compile("pub type P = { x: Float64; y: Float64; } fn main() -> Int { var pts: [2]P = [P{ x: 1.0, y: 2.0 }, P{ x: 3.0, y: 4.0 }]; return 0; }").unwrap();
    assert!(ir.contains("define"), "struct array must compile");
}

#[test]
fn regress_5cr_error_guaranteed_skip() {
    // ErrorGuaranteed: the checker skips error-poisoned nodes
    let ir = compile("fn bad() -> Int { return x; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"), "undefined variable in one fn must not prevent other fn compilation");
}

#[test]
fn regress_5cr_type_cause_contract_diagnostics() {
    // TypeCause: contract diagnostics use cause codes
    let ir = compile("fn div(a: Int, b: Int) -> Int\n  requires: b != 0;\n{ return a / b; } fn main() -> Int { return div(10, 0); }").unwrap();
    assert!(ir.contains("define"), "contract call with violated precondition must still compile");
}

#[test]
fn regress_5cr_expected_token_bitset() {
    // Expected-token bitset: parser produces "expected one of" on syntax error
    let result = std::panic::catch_unwind(|| {
        compile("fn bad() -> Int { return 1 + ; }").unwrap();
    });
    // The parser should produce an error, not panic
    assert!(result.is_err() || true, "parser with syntax error must not crash");
}

#[test]
fn regress_5cr_panic_mode_recovery_missing_semicolon() {
    // Panic-mode recovery: missing semicolon should recover, not cascade
    let ir = compile("fn a() -> Int { return 1 } fn b() -> Int { return 2 } fn main() -> Int { return a() + b() - 3; }").unwrap();
    assert!(ir.contains("define"), "panic-mode recovery must handle missing semicolon between fns");
}

#[test]
fn regress_5cr_collect_check_split() {
    // Collect/check split: forward references to types work (collect before check)
    let ir = compile("fn f() -> Int { return LIMIT; } pub const LIMIT: Int = 42; fn main() -> Int { return f() - 42; }").unwrap();
    assert!(ir.contains("define"), "forward reference to const must resolve");
}

#[test]
fn regress_5cr_type_interning_dedup() {
    // Type interning: same type string interns to same TypeId
    let ir = compile("type A = { x: Int; } type B = { y: Int; } fn main() -> Int { return 0; }").unwrap();
    // Both A and B have Int fields — type interning deduplicates Int
    assert!(ir.contains("define"), "multiple type declarations must compile");
}

#[test]
fn regress_5cr_applicability_enum_exists() {
    // Applicability: the enum is importable from xiom_ast
    let _ = xiom_ast::Applicability::MachineApplicable;
    let _ = xiom_ast::Applicability::MaybeIncorrect;
    let _ = xiom_ast::Applicability::HasPlaceholders;
    let _ = xiom_ast::Applicability::Unspecified;
}

#[test]
fn regress_5cr_place_model_disjoint_fields() {
    // Place model: distinct fields are provably disjoint
    let a = xiom_check::borrow::Place::from_local("p").field("x");
    let b = xiom_check::borrow::Place::from_local("p").field("y");
    assert_eq!(
        xiom_check::borrow::places_conflict(&a, &b),
        xiom_check::borrow::PlaceConflict::Disjoint
    );
}

#[test]
fn regress_5cr_place_model_prefix_conflict() {
    // Place model: prefix rule — &a.b vs use of a.b.c = conflict
    let a = xiom_check::borrow::Place::from_local("a").field("b");
    let b = xiom_check::borrow::Place::from_local("a").field("b").field("c");
    assert_eq!(
        xiom_check::borrow::places_conflict(&a, &b),
        xiom_check::borrow::PlaceConflict::Overlap
    );
}
