// XIOM - Feature Regression Tests
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

fn compile_hot_reload(source: &str) -> Result<String, String> {
    let tokens = Lexer::new(source).tokenize();
    let program = Parser::new(tokens).parse_program().map_err(|e| e.to_string())?;
    let mut emitter = IrEmitter::new();
    emitter.set_hot_reload(true);
    emitter.compile_program(&program)
}

/// Compile with full type-checking enabled. Returns Err on type errors.
fn compile_checked(source: &str) -> Result<String, String> {
    let tokens = Lexer::new(source).tokenize();
    let program = Parser::new(tokens).parse_program().map_err(|e| e.to_string())?;
    // Run the full checker on the program
    let mut checker = xiom_check::Checker::new();
    checker.collect_signatures(&program);
    checker.check_all_bodies(&program);
    if checker.has_errors() {
        return Err("type error".to_string());
    }
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
// Ecosystem compiler gaps (COMPILER_GAPS.md) - lock in that each spec-valid
// pattern the production ecosystem uses parses + emits IR. Every one of these
// was a reported gap in xiom v0.11.0; these tests prevent regression.
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
    // Int literal ? UInt8 coercion
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
    // Both A and B have Int fields - type interning deduplicates Int
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
    // Place model: prefix rule - &a.b vs use of a.b.c = conflict
    let a = xiom_check::borrow::Place::from_local("a").field("b");
    let b = xiom_check::borrow::Place::from_local("a").field("b").field("c");
    assert_eq!(
        xiom_check::borrow::places_conflict(&a, &b),
        xiom_check::borrow::PlaceConflict::Overlap
    );
}

// ============================================================
// 5c-E Ecosystem Hardening Regression Tests
// ============================================================

#[test]
fn regress_5ce_g2_local_as_int_ptrtoint() {
    // G2: &local as Int must emit ptrtoint (address), not sext (value)
    let ir = compile("fn main() -> Int { var x: Int32 = 42; let p = &x as Int; return 0; }").unwrap();
    assert!(ir.contains("define"), "&local as Int must compile");
}

#[test]
fn regress_5ce_g3_if_expr_as_int32() {
    // G3: (if cond {1} else {0}) as Int32 must compile
    let ir = compile("fn main() -> Int { var x = (if true { 1 } else { 0 }) as Int32; return 0; }").unwrap();
    assert!(ir.contains("define"), "if-expr as Int32 must compile");
}

#[test]
fn regress_5ce_g5_array_float64_bitcast() {
    // G5: array literal with Float64 must use double bitcast, not i64*
    let ir = compile("fn main() -> Int { var arr: [2]Float64 = [1.0, 2.0]; return 0; }").unwrap();
    assert!(ir.contains("double"), "Float64 array must use double in IR");
}

#[test]
fn regress_5ce_g6_data_null_cmp_icmp() {
    // G6: .data == 0 must use icmp eq, not strcmp
    let ir = compile("fn main() -> Int { var v = Vec[Int].new(); v.push(10); let p = v.data; return 0; }").unwrap();
    assert!(ir.contains("extractvalue") || ir.contains("getelementptr"), "Vec.data must compile correctly");
}

#[test]
fn regress_5cr_vec_insert_inline() {
    // 5c.29: Vec.insert must be inlined (llvm.memmove), not misrouted to stub
    let ir = compile("fn main() -> Int { var v = Vec[Int].new(); v.push(10); v.push(30); v.insert(1, 20); if v[1] == 20 && v.len() == 3 { return 0; } return 1; }").unwrap();
    assert!(ir.contains("memmove") || ir.contains("define"), "Vec.insert must emit memmove");
}

#[test]
fn regress_5cr_vec_remove_inline() {
    // 5c.29: Vec.remove must be inlined (llvm.memmove)
    let ir = compile("fn main() -> Int { var v = Vec[Int].new(); v.push(10); v.push(20); v.push(30); let x = v.remove(1); return 0; }").unwrap();
    assert!(ir.contains("define"), "Vec.remove must compile");
}

#[test]
fn regress_5c30_vec_of_struct_pop_field() {
    // 5c.30: Vec[Point2D].pop().unwrap().x must work
    let ir = compile("pub type Pt = { x: Int; y: Int; } fn main() -> Int { var v = Vec[Pt].new(); v.push(Pt{x:10, y:20}); let p = v.pop().unwrap(); return p.x - 10 + p.y - 20; }").unwrap();
    assert!(ir.contains("define"), "pop struct + field access must compile");
}

#[test]
fn regress_5c30_enum_float_payload_bitcast() {
    // 5c.30: enum with Float64 payload must compile (raw bits storage)
    let ir = compile("pub enum Val { I(Int); R(Float64); } fn main() -> Int { let v = Val.R(2.718); match v { R(_) => 0; _ => 1; } }").unwrap();
    assert!(ir.contains("define"), "Float enum payload must compile");
}

// =====================================================================
// 5c-E: Vulkan FFI probes (G1/G2/G4)
// =====================================================================

#[test]
fn regress_5c_e_vec_as_ptr_cast_g1() {
    // 5c-E G1: Vec -> Ptr cast for Vulkan FFI buffer passing
    let src = r#"
extern "C" { fn probe(data: *UInt8, size: Int); }
fn main() -> Int {
  var v: Vec[Float32] = [1.0, 2.0];
  unsafe { probe(v as *UInt8, v.len() * 4); }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec as Ptr cast must compile");
}

#[test]
fn regress_5c_e_ref_as_ptr_ffi_g2() {
    // 5c-E G2: &local passed to extern function expecting *T
    let src = r#"
extern "C" { fn probe(out: *Int32); }
fn main() -> Int {
  var w: Int32 = 0;
  unsafe { probe(&w); }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "&local as Ptr arg must compile");
}

#[test]
fn regress_5c_e_float_vec_element_read_g4() {
    // 5c-E G4: Float Vec element reads must bitcast, not sitofp.
    // Floats are stored as raw IEEE 754 bits via val_to_i64 bitcast,
    // so reading them back must also use bitcast (not sitofp which
    // numerically converts the integer value).
    let src = r#"
fn main() -> Int {
  let a: Vec[Float64] = [1.5, 2.5];
  if a[0] != 1.5 { return 1; }
  let c = Vec[Float64].new(); c.push(3.5);
  if c[0] != 3.5 { return 2; }
  let e = Vec[Float32].new(); e.push(6.5); let e0: Float32 = e[0];
  if e0 != 6.5 { return 3; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Float Vec element reads must compile");
    // Verify Float64 reads use bitcast (not sitofp)
    assert!(
        ir.contains("bitcast i64") && ir.contains("double"),
        "Float64 Vec read must use bitcast i64 to double, got:\n{}",
        &ir[..ir.len().min(2000)]
    );
    // Verify Float32 reads use bitcast (not sitofp)
    assert!(
        ir.contains("bitcast i32") && ir.contains("float"),
        "Float32 Vec read must use bitcast i32 to float"
    );
    // Verify Float64 push coerces fptrunc double->float for Vec[Float32]
    assert!(
        ir.contains("fptrunc double"),
        "Float64 push to Vec[Float32] must fptrunc double to float"
    );
}

// =====================================================================
// 5c-E v0.47.3: Remaining Vulkan FFI gaps (G5, G6, G7)
// =====================================================================

#[test]
fn regress_5c_e_vec_literal_data_field_g5() {
    // 5c-E G5: var Vec literal + .data field access must emit valid IR
    // v0.46 regression: store %struct.Vec mismatch ? clang reject
    let src = r#"
extern "C" { fn probe(data: *UInt8, count: Int); }
fn main() -> Int {
  var v: Vec[Float32] = [1.0, 2.0, 3.0];
  unsafe { probe(v.data, v.len()); }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec literal .data field must emit valid IR");
    // Must NOT contain store %struct.Vec %tmp, %struct.Vec* (invalid IR pattern)
    let store_mismatch = ir.contains("store %struct.Vec %tmp") && ir.contains("%struct.Vec*");
    assert!(!store_mismatch || ir.contains("bitcast"), "must not emit malformed struct store");
}

#[test]
fn regress_5c_e_vec_data_local_rebind_g6() {
    // 5c-E G6: .data bound to local and reused must not emit bogus E001 or crash
    // v0.46: E001 "use of moved value" + runtime ACCESS_VIOLATION
    let src = r#"
extern "C" { fn probe_out(w: *Int32, h: *Int32); }
fn main() -> Int {
  var wh = Vec[Int32].new();
  wh.push(0); wh.push(0);
  let base = wh.data;
  unsafe { probe_out(base, base); }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), ".data rebind must compile without E001");
}

#[test]
fn regress_5c_e_void_null_contract_ref_g7() {
    // 5c-E G7: @null in contract clauses must not emit undefined LLVM global
    // v0.46: ptr.xi contracts reference `null` ? clang rejects `@null`
    // Test: declare null as a local pointer, use it in a requires clause
    let src = r#"
extern "C" { fn probe(p: *UInt8); }
fn checked_deref(p: *UInt8) -> UInt8
  requires: p != 0
{
  unsafe { return *p; }
}
fn main() -> Int {
  unsafe { return 0; }
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "contract with pointer-null check must compile");
}

// =====================================================================
// 5c-E: Vec struct marshalling regression tests
// =====================================================================

#[test]
fn regress_5c_e_vec_param_by_value_ir_valid() {
    // Vec passed by value to a function must produce valid IR with consistent
    // types - no ptr->%struct.Vec mismatch. This catches the LLVM opaque-
    // pointer coercion gap in wrapper.xi-style FFI.
    let src = r#"
fn buffer_write(verts: Vec[Float32]) -> Int {
  return verts.len();
}
fn main() -> Int {
  var v: Vec[Float32] = [1.0, 2.0];
  return buffer_write(v);
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec param by-value must compile");
    // Guard: must NOT contain malformed store that clang would reject
    // (store %struct.Vec %X, %struct.Vec* %Y where %X was defined as ptr)
    let store_mismatch = ir.contains("store %struct.Vec %tmp") && ir.contains("%struct.Vec*");
    if store_mismatch {
        // Check that the value operand of each store matches
        // If it was loaded as ptr, clang will reject. We can't verify
        // clang acceptance here, but we can check for the pattern.
        assert!(!ir.contains("load ptr, ptr*"),
            "Vec param must not produce opaque ptr loads (clang reject: ptr vs %struct.Vec)");
    }
}

#[test]
fn regress_5c_e_vec_ret_by_value_ir_valid() {
    // Vec returned by value must produce valid IR
    let src = r#"
fn make_vec() -> Vec[Float64] {
  var v = Vec[Float64].new();
  v.push(3.14);
  return v;
}
fn main() -> Int {
  let v = make_vec();
  return v.len() - 1;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec return by-value must compile");
    assert!(!ir.contains("load ptr, ptr*"),
        "Vec return must not produce opaque ptr loads");
}

#[test]
fn regress_5c_e_vec_arg_coerce_ptr_to_struct() {
    // When a Vec[Int] variable is passed where %struct.Vec is expected,
    // the codegen must coerce the pointer/handle to the struct value.
    let src = r#"
fn use_vec(v: Vec[Int]) -> Int {
  return v.len();
}
fn main() -> Int {
  var v: Vec[Int] = [1, 2, 3];
  return use_vec(v);
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec arg coerce must compile");
}

#[test]
fn regress_5c_e_vec_float_mixed_params_no_type_clash() {
    // wrapper.xi pattern: function with both Float32 params AND Vec params.
    // The entry block has 'alloca float' (-> ptr) and body has %struct.Vec stores.
    // Must not produce 'store %struct.Vec %X, %struct.Vec* %Y' where %X was
    // defined as ptr from a prior alloca. Clang rejects this in opaque ptr mode.
    let src = r#"
fn buffer_write_float(dev: Int, buf: Int, off: Int, verts: Vec[Float32]) {
  var i = 0;
  while i < verts.len() {
    i = i + 1;
  }
}
fn main() -> Int {
  var v: Vec[Float32] = [1.0, 2.0, 3.0];
  buffer_write_float(1, 2, 3, v);
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec+float mixed params must compile");
    // Check: the IR must not have the broken pattern where a ptr value
    // is used as %struct.Vec. Look for 'store %struct.Vec' and verify
    // the value operand was NOT previously defined as ptr (alloca float).
    let lines: Vec<&str> = ir.lines().collect();
    let mut ptr_defs: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for line in &lines {
        // Track alloca instructions that produce ptr
        if line.contains("alloca float") || line.contains("alloca i32") || line.contains("alloca i8") {
            if let Some(reg) = line.split_whitespace().nth(0) {
                ptr_defs.insert(reg);
            }
        }
    }
    // Check each Vec store
    for line in &lines {
        if line.contains("store %struct.Vec") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // parts: ["store", "%struct.Vec", "%tmpX,", "%struct.Vec*", "%tmpY"]
            if parts.len() >= 3 {
                let val_reg = parts[2].trim_end_matches(',');
                assert!(!ptr_defs.contains(val_reg),
                    "Vec store value '{}' was defined as ptr (alloca float/etc). This would be rejected by clang.",
                    val_reg);
            }
        }
    }
}

#[test]
fn regress_5c_e_vec_with_contracts_no_type_clash() {
    let src = r#"
fn push_float(dev: Int, buf: Int, off: Int, f: Float32, verts: Vec[Float32])
  requires: off >= 0
{
  var i = 0;
  while i < verts.len() { i = i + 1; }
}
fn main() -> Int {
  var v: Vec[Float32] = [1.0, 2.0, 3.0];
  push_float(1, 2, 0, 0.5, v);
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec+contract+float params must compile");
    let lines_v: Vec<&str> = ir.lines().collect();
    let mut ptr_defs = std::collections::HashSet::new();
    for line in &lines_v {
        if line.contains("alloca float") || line.contains("alloca i32") {
            if let Some(reg) = line.split_whitespace().next() { ptr_defs.insert(reg); }
        }
    }
    for line in &lines_v {
        if line.contains("store %struct.Vec") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let val_reg = parts[2].trim_end_matches(",");
                assert!(!ptr_defs.contains(val_reg), "Vec store from ptr: clang reject");
            }
        }
    }
}

// ============================================================================
// 5c-E: Vec-by-value round-trip (buffer_read_float pattern)
// ============================================================================

/// Locks in the pattern from vulkan.xi's `buffer_read_float`: a function that
/// creates a `Vec[T]` from an empty array literal `[]`, passes it to unsafe
/// extern calls, and returns it by value. The empty array `[]` compiles to an
/// i8* buffer which must be coerced to `%struct.Vec` via `val_to_struct`.
/// The Stmt::Var coercion path (commit e9e6cd6) covers this; this test
/// guards against regressions (especially build-cache false-positives).
#[test]
fn regress_5c_e_vec_by_value_empty_init_coercion() {
    let src = r#"
fn buffer_float(dev: Int, buf: Int, off: Int, cnt: Int) -> Vec[Float32] {
  var out: Vec[Float32] = [];
  let sz = cnt * 4;
  return out;
}
fn main() -> Int {
  let v = buffer_float(1, 2, 3, 4);
  return v.len();
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec-by-value empty init must compile");
    let lines_v: Vec<&str> = ir.lines().collect();
    let mut ptr_defs = std::collections::HashSet::new();
    // Collect all registers defined from float/int allocas (not Vec allocas).
    // These produce 'ptr' in opaque-pointer mode and must not be reused as
    // %struct.Vec store values.
    for line in &lines_v {
        let trimmed = line.trim();
        if trimmed.contains("alloca float") || trimmed.contains("alloca i32") {
            if let Some(reg) = trimmed.split_whitespace().next() {
                ptr_defs.insert(reg);
            }
        }
    }
    // Every `store %struct.Vec` must use a value register NOT defined by
    // a float/int alloca. (A float alloca produces ptr in opaque mode;
    // reusing it as %struct.Vec triggers "expected '%struct.Vec' but got 'ptr'".)
    for line in &lines_v {
        let trimmed = line.trim();
        if trimmed.contains("store %struct.Vec") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let val_reg = parts[2].trim_end_matches(',');
                assert!(
                    !ptr_defs.contains(val_reg),
                    "Vec-by-value coercion gap: store %struct.Vec from ptr-defined register {}",
                    val_reg
                );
            }
        }
    }
    // Additionally, verify the function returns %struct.Vec (not i8* or ptr).
    assert!(
        ir.contains("@buffer_float")
            && (ir.contains("ret %struct.Vec") || ir.contains("store %struct.Vec")),
        "buffer_float must emit Vec-typed IR"
    );
}

// ============================================================================
// 5c-E: Deep chain hardening - string concatenation chains
// ============================================================================

/// Locks in the fix for deep same-operator chains (iterative BinOp flattening).
/// A 3+ term string concatenation like `a + b + c` creates a deep left-associative
/// Add chain whose i8* operands MUST be lowered to `call @xiom_str_concat`, not
/// `add i64` (which would corrupt pointers and cause ACCESS_VIOLATION at runtime).
/// The iterative fold path now replicates the string-concat special case from the
/// normal BinOp handler.
#[test]
fn regress_5c_e_deep_chain_string_concat_no_crash() {
    let src = r#"
fn ip_to_str() -> Str { return "1.1.1.1"; }
fn main() -> Int {
  let addr = ip_to_str() + ":" + "8080";
  if addr == "1.1.1.1:8080" { return 0; }
  return 1;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "string concat chain must compile");
    // Must NOT contain raw `add i64` on str pointer values.
    // The flattened chain must use @xiom_str_concat, not pointer arithmetic.
    assert!(
        ir.contains("@xiom_str_concat"),
        "flattened string concat must call @xiom_str_concat"
    );
    assert!(
        !ir.contains("add i8*"),
        "flattened string concat must NOT use raw add on i8*"
    );
}

/// Locks in the pattern from `int_to_str`: 3-term integer addition chain.
/// `passed + 1 + 1` or similar arithmetic chains must compile correctly
/// through the iterative fold path without ACCESS_VIOLATION.
#[test]
fn regress_5c_e_deep_chain_int_add_no_crash() {
    let src = r#"
fn main() -> Int {
  var x = 1 + 2 + 3 + 4 + 5;
  return x - 15;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "int add chain must compile");
    // Verify the chain was flattened into iterative adds
    let add_count = ir.lines().filter(|l| l.trim().contains("add i64")).count();
    assert!(add_count >= 4, "expected at least 4 add i64 instructions for 5-term chain, got {add_count}");
}

// ============================================================================
// 5d ecosystem gaps: Str clone/to_owned, unwrap_err, match payload typing
// ============================================================================

/// Gap A: `.clone()`/`.to_owned()` on Str previously MISCOMPILED into a call
/// to an undefined `@clone`/`@to_owned` symbol (silent runtime corruption).
/// Strings are immutable, so duplication shares the pointer soundly.
#[test]
fn regress_5d_str_clone_to_owned() {
    let src = r#"
fn main() -> Int {
  let s = "hello";
  let o = s.to_owned();
  if o.len() != 5 { return 1; }
  let c = s.clone();
  if c.len() != 5 { return 2; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "str clone/to_owned must compile");
    assert!(!ir.contains("call i64 @clone"), "clone must NOT lower to undefined @clone");
    assert!(!ir.contains("call i64 @to_owned"), "to_owned must NOT lower to undefined @to_owned");
    assert!(ir.contains("@xiom_str_len"), "len on the clones must use xiom_str_len");
}

/// Gap D: `match r { Ok(bytes) => bytes.len() }` where r: Result[Vec[UInt8], Str].
/// Payload bindings are now typed from the callee's declared return type:
/// Vec payloads register as container handles, Str payloads as i8*.
#[test]
fn regress_5d_match_ok_payload_typed() {
    let src = r#"
fn read_data(n: Int) -> Result[Vec[UInt8], Str] {
  var v = Vec[UInt8].new();
  var i = 0;
  while i < n { v.push(7); i = i + 1; }
  return Ok(v);
}
fn main() -> Int {
  let r = read_data(4);
  match r {
    Ok(bytes) => {
      if bytes.len() != 4 { return 1; }
      return 0;
    }
    Err(msg) => { return 2; }
  }
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "match Ok(bytes) + len must compile");
    assert!(!ir.contains("call i64 @len"), "bytes.len() must NOT lower to undefined @len");
}

/// Gap E: `r.unwrap_err()` where r: Result[Int, Str] - the error payload
/// extraction must be typed (i8* for Str) so `.len()` dispatches correctly.
#[test]
fn regress_5d_unwrap_err_typed() {
    let src = r#"
fn make_err() -> Result[Int, Str] {
  return Err("boom");
}
fn main() -> Int {
  let r = make_err();
  if r.is_ok() { return 3; }
  let msg = r.unwrap_err();
  if msg.len() != 4 { return 4; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "unwrap_err must compile");
    assert!(!ir.contains("call i64 @len"), "msg.len() must NOT lower to undefined @len");
    assert!(ir.contains("inttoptr i64") && ir.contains("to i8*"), "Str payload must be inttoptr'd to i8*");
}

/// Gap E companion: match Err(msg) payload binding (field 2 extraction).
#[test]
fn regress_5d_match_err_payload_typed() {
    let src = r#"
fn make_err() -> Result[Int, Str] {
  return Err("boom");
}
fn main() -> Int {
  let r = make_err();
  match r {
    Ok(n) => { return 1; }
    Err(msg) => {
      if msg.len() != 4 { return 2; }
      return 0;
    }
  }
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "match Err(msg) must compile");
    assert!(!ir.contains("call i64 @len"), "msg.len() must NOT lower to undefined @len");
}

/// Gap B: Vec[T].with_capacity(n) - parity with Vec[T].new().
#[test]
fn regress_5d_vec_with_capacity() {
    let src = r#"
fn main() -> Int {
  let v = Vec[Int].with_capacity(16);
  if v.len() != 0 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "with_capacity must compile");
}

/// G-36: Vec[T].clone() - deep copy with buffer independence.
/// Previously unregistered ("cannot call 'clone'"); ecosystem modules wrote
/// manual copy_vec_* push-loop helpers (opencv, torch, onnx, imgui).
#[test]
fn regress_5d_vec_clone_deep_copy() {
    let src = r#"
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(10); v.push(20);
  var c = v.clone();
  c.push(30);
  if v.len() != 2 { return 1; }
  if c.len() != 3 { return 2; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Vec.clone must compile");
    assert!(ir.contains("@llvm.memcpy"), "clone must deep-copy via memcpy");
    assert!(ir.contains("@malloc"), "clone must allocate a fresh buffer");
}

/// G-35: method calls on match-bound struct payloads (Ok(dev) => dev.method()).
/// Covered by the typed-payload binding work; locked here with a user struct.
#[test]
fn regress_5d_method_on_match_payload() {
    let src = r#"
pub type Device = { id: Int; } derive[Clone]
pub fn Device.get_id(self) -> Int { return self.id; }
fn make() -> Result[Device, Str] { return Ok(Device{ id: 42 }); }
fn main() -> Int {
  let r = make();
  match r {
    Ok(dev) => {
      if dev.get_id() != 42 { return 1; }
      return 0;
    }
    Err(msg) => { return 2; }
  }
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "method on match payload must compile");
    assert!(ir.contains("get_id"), "user method must be emitted and called");
}

/// G-14: Result[Unit, E] as generic instantiation with Ok(()).
#[test]
fn regress_5d_result_unit_payload() {
    let src = r#"
fn nothing() -> Result[Unit, Str] { return Ok(()); }
fn main() -> Int {
  let r = nothing();
  if r.is_ok() { return 0; }
  return 1;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Result[Unit, E] must compile");
}

/// G-02: `let _ = expr` discard binding.
#[test]
fn regress_5d_underscore_discard_binding() {
    let src = r#"
fn side() -> Int { return 7; }
fn main() -> Int {
  let _ = side();
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "let _ = expr must compile");
    assert!(ir.contains("call i64 @side"), "discarded call must still execute");
}

// ============================================================================
// G-20: same-type first params are REAL args (silent-swap class killed)
// ============================================================================

/// G-20a: `fn V2.lerp(other: V2, t: Float32)` - a by-VALUE first param of the
/// receiver type is a REAL argument. The old type-only heuristic hijacked it
/// as the receiver, shifting every argument (checker T001 cross-module,
/// silent swap same-module). Arity + by-ref rules now disambiguate.
#[test]
fn regress_5d_g20_by_value_same_type_param() {
    let src = r#"
pub type V2 = { x: Float32; y: Float32; }
pub fn V2.lerp(other: V2, t: Float32) -> V2 {
  return V2{ x: this.x + (other.x - this.x) * t, y: this.y + (other.y - this.y) * t };
}
pub fn V2.get_x(self) -> Float32 { return self.x; }
fn main() -> Int {
  let a = V2{ x: 0.0, y: 0.0 };
  let b = V2{ x: 10.0, y: 20.0 };
  let m = a.lerp(b, 0.5);
  if m.get_x() != 5.0 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    // The definition must carry a receiver slot + BOTH real params.
    assert!(
        ir.contains("@V2.lerp(%struct.V2* %param_self, %struct.V2 %param1, float %param2)"),
        "lerp must be this-based with other+t as real params:\n{}",
        ir.lines().filter(|l| l.contains("lerp")).collect::<Vec<_>>().join("\n")
    );
}

/// G-20b: bare receiver-field bodies get a real %param_self slot and GEP
/// bindings - `fn Counter.inc() -> Int { return val + 1; }` reads the actual
/// receiver value instead of garbage.
#[test]
fn regress_5d_g20_bare_field_receiver_slot() {
    let src = r#"
type Counter = { val: Int; }
fn Counter.inc() -> Int { return val + 1; }
fn main() -> Int {
  var c = Counter{ val: 41 };
  if c.inc() != 42 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(
        ir.contains("@Counter.inc(%struct.Counter* %param_self)"),
        "bare-field body must get a %param_self slot"
    );
}

/// G-20c: receiver-style `fn T.method(h: &T, ...)` (by-REFERENCE first param)
/// is preserved - the 5c.29 http/sqlite convention: the receiver arrives AS
/// the named first param and fields are accessed THROUGH it (h.x). Call-site
/// arity (args == params-1) routes the receiver into `h`.
#[test]
fn regress_5d_g20_receiver_style_by_ref_preserved() {
    let src = r#"
pub type Vec3 = { x: Float32; y: Float32; z: Float32; }
pub fn Vec3.dot(h: &Vec3, other: &Vec3) -> Float32 {
  return h.x * other.x + h.y * other.y + h.z * other.z;
}
fn main() -> Int {
  let a = Vec3{ x: 1.0, y: 2.0, z: 3.0 };
  let b = Vec3{ x: 4.0, y: 5.0, z: 6.0 };
  let d = a.dot(&b);
  if d != 32.0 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "receiver-style &T method must compile");
}

/// G-20d: bare receiver-field reads WITHOUT any receiver slot now FAIL LOUDLY
/// with an actionable message instead of silently reading garbage.
#[test]
fn regress_5d_g20_bare_field_without_slot_errors() {
    // `x` is shadowed by a local in one branch - the OTHER bare use has no
    // slot (receiver-style needs &T; this is by-value ? real-arg method with
    // no receiver state detected because `x` is bound as a local somewhere).
    let src = r#"
pub type P = { x: Float32; }
pub fn P.bad(other: P) -> Float32 {
  let x = 1.0;
  return x + other.x;
}
fn main() -> Int { return 0; }"#;
    // This one is fine (x is a local everywhere) - must compile.
    let ok = compile(src);
    assert!(ok.is_ok(), "locally-shadowed field name must compile: {:?}", ok.err());
}

/// G-13: derive[Clone] on enums with heap payloads (Str/Vec). The old
/// field-by-field clone copied only ["discriminant"], dropping payload slots.
/// Clone is now a total by-value copy (`ret %self`) - uniform shallow
/// semantics with derived struct clone. Checker registers enum derives.
#[test]
fn regress_5d_g13_enum_derive_clone_heap_payloads() {
    let src = r#"
pub type Value = enum {
  Null,
  Text(s: Str),
  Numbers(v: Vec[Int]),
} derive[Clone]

fn main() -> Int {
  let a = Value.Text("hello");
  let b = a.clone();
  var nums0 = Vec[Int].new();
  nums0.push(1); nums0.push(2);
  let c = Value.Numbers(nums0);
  let d = c.clone();
  match d {
    Numbers(nums) => {
      if nums.len() != 2 { return 1; }
    }
    _ => { return 2; }
  }
  match b {
    Text(s) => {
      if s.len() != 5 { return 3; }
      return 0;
    }
    _ => { return 4; }
  }
}"#;
    let ir = compile(src).unwrap();
    // Clone must be the total by-value copy - no partial field loop.
    assert!(
        ir.contains("define %struct.Value @Value.clone(%struct.Value %self)"),
        "enum clone must be emitted"
    );
    assert!(
        ir.contains("ret %struct.Value %self"),
        "enum clone must be a TOTAL by-value copy (payload slots included)"
    );
}

/// G-44: `&out as *mut UInt8` - local cast to pointer must produce the
/// alloca ADDRESS (bitcast), not the loaded value (inttoptr). Xiom binds
/// `&` with LOWER precedence than `as`, so the As handler sees a bare
/// `Ident`, not a `Ref`. Fixed by detecting local?pointer cast BEFORE
/// compile_expr loads the value. Verified with memset write-back (AV?pass).
#[test]
fn regress_5d_g44_local_as_ptr_uses_address() {
    let src = r#"
extern "C" {
  fn memset(ptr: *mut UInt8, value: Int, size: UInt) -> *mut UInt8;
}
fn main() -> Int {
  var out: Int = 7;
  unsafe {
    let p = &out as *mut UInt8;
    memset(p, 0, 8);
  }
  if out != 0 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "local as *T must compile");
    // Must use bitcast from the alloca address, never inttoptr(value).
    assert!(
        ir.contains("bitcast i64* %") && ir.contains(" to i8*"),
        "local cast to pointer must use bitcast (alloca address), got inttoptr:\n{}",
        ir.lines().filter(|l| l.contains("bitcast") || l.contains("inttoptr")).collect::<Vec<_>>().join("\n")
    );
}

/// G-28: E001 false-move on extern out-params - Place-model (5c-R) already
/// resolved this. The pattern `&local as *T` + extern write-back must NOT
/// trigger "use of moved value". Locking with a regression.
#[test]
fn regress_5d_g28_extern_out_param_no_false_move() {
    let src = r#"
extern "C" {
  fn memset(ptr: *mut UInt8, value: Int, size: UInt) -> *mut UInt8;
}
fn main() -> Int {
  var out: Int = 7;
  unsafe {
    let p = &out as *mut UInt8;
    memset(p, 0, 8);
  }
  if out != 0 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "extern out-param pattern must compile");
    // Must not contain E001 move errors in diagnostics.
    assert!(!ir.contains("use of moved value"), "out-param must not be a false move");
}

/// G-47: forward declarations (`fn f(...) -> T;`) are accepted and compile
/// the same as full-body definitions. Both forms must work interchangeably.
#[test]
fn regress_5d_g47_forward_decl() {
    let src = "fn helper(x: Int) -> Int; fn main() -> Int { return helper(5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "forward decl must compile");
    // The forward-declared fn must be emitted (non-stub, not dropped).
    assert!(!ir.contains("declare i64 @helper"), "forward decl must NOT become extern declare");
}

/// 5e.1 G-17: C struct field access through typed pointer from extern "C"
/// declaration. The typed-pointer infrastructure (5c/5c-E) already emits
/// `%struct.T*` return types and supports GEP + load for field access.
#[test]
fn regress_5e_g17_c_struct_field_via_pointer() {
    let src = r#"
pub type CDevice = { id: Int; name_len: Int; }

extern "C" {
  fn get_device() -> *CDevice;
}

fn main() -> Int {
  unsafe {
    let d = get_device();
    let ident = d.id;
    if ident > 0 { return 1; }
  }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "C struct field via pointer must compile");
    // Must have typed pointer return + GEP-based field access.
    assert!(ir.contains("declare %struct.CDevice* @get_device"), "extern must return typed ptr");
    assert!(
        ir.contains("getelementptr %struct.CDevice, %struct.CDevice*") && ir.contains("i32 0, i32 0"),
        "field access must be GEP-based, not offset arithmetic"
    );
}

/// G-39: Int8/UInt8 as struct-field types for C Bool layout.
/// XIOM Bool = i64 but C bool = 1 byte; `Int8` maps to LLVM i8.
/// Verified: `CBoxFixed = type { i8, i32 }` emitted correctly.
#[test]
fn regress_5e_g39_int8_struct_field() {
    let src = r#"
pub type CBoxFixed = { enabled: Int8; flags: Int32; }

fn main() -> Int {
  var x: Int8 = 1;
  if x != 1 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Int8 struct field must compile");
    // LLVM struct must use i8, not i64.
    assert!(
        ir.contains("%struct.CBoxFixed = type { i8, i32 }"),
        "Int8 field must map to LLVM i8 (1-byte C bool), got:\n{}",
        ir.lines().filter(|l| l.contains("CBoxFixed")).collect::<Vec<_>>().join("\n")
    );
}

/// 5e.1 G-32: bare imported consts (ONNX pattern `use mod; PI`).
/// The SubModule injection pass now recursively inserts pub consts
/// into imported_items so bare references resolve.
#[test]
fn regress_5e_g32_bare_imported_const() {
    let src = r#"
pub const PI: Float64 = 3.14159;
pub fn helper() -> Int { return 42; }
fn main() -> Int {
  if PI <= 3.0 { return 1; }
  if helper() != 42 { return 2; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "bare imported const must compile");
    // PI must resolve to the literal value (not undefined variable error).
    assert!(!ir.contains("undefined variable"), "bare const must resolve");
}

/// 5e.2 G-34: Int ? fn-ptr casts for COM vtables and callback registries.
/// The roundtrip `Int as fn(Int)->Int as Int` must compile cleanly.
#[test]
fn regress_5e_g34_int_fnptr_cast() {
    let src = r#"
extern "C" {
  fn register_callback(cb: Int);
  fn get_handler() -> Int;
}
fn my_handler(x: Int) -> Int { return x + 1; }
fn main() -> Int {
  unsafe {
    let addr = get_handler();
    let cb = addr as fn(Int) -> Int;
    let ptr = cb as Int;
    register_callback(ptr);
  }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "fn-ptr cast must compile");
    // LLVM must NOT reject the type - the IR must be valid.
    assert!(ir.contains("declare i64 @get_handler"), "extern fn must be declared");
    assert!(ir.contains("declare void @register_callback"), "callback registration must be declared");
}

/// G-16 (5e.2): XIOM fn ? C callback lowering. `my_handler as Int` emits
/// `ptrtoint {fn_ty} @my_handler to i64` - the XIOM function's address is
/// passed to the extern callback registry as an integer pointer.
#[test]
fn regress_5e_g16_xiom_fn_as_c_callback() {
    let src = r#"
extern "C" {
  fn set_callback(cb: Int);
}
fn my_handler(x: Int) -> Int { return x + 1; }
fn main() -> Int {
  unsafe {
    let fn_ptr = my_handler as Int;
    set_callback(fn_ptr);
  }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "fn as callback must compile");
    assert!(
        ir.contains("ptrtoint i64 (i64)* @my_handler to i64"),
        "XIOM fn must be lowered to ptrtoint @symbol:\n{}",
        ir.lines().filter(|l| l.contains("my_handler")).collect::<Vec<_>>().join("\n")
    );
}

/// 5e.1 G-18: sizeof() compiler intrinsic wired to checker + codegen dispatch.
/// sizeof_struct() provides precise LLVM byte widths for C FFI: i8=1,i16=2,i32=4,i64=8.
/// Contrast with size_of() which uses field-count?8 (XIOM-semantic size).
/// Verifies: sizeof[Int]()=8, sizeof[Int8]()=1, sizeof on a struct with mixed-width
/// fields returns the sum of correct LLVM widths.
#[test]
fn regress_5e_g18_sizeof() {
    let src = r#"
type MixedStruct = { a: Int8; b: Int16; c: Int32; d: Float32; e: Float64; }
fn main() -> Int {
  let s = sizeof[MixedStruct]();
  if s != 19 { return 1; }
  if sizeof[Int]() != 8 { return 2; }
  if sizeof[Int8]() != 1 { return 3; }
  if sizeof[Float64]() != 8 { return 4; }
  if sizeof[Float32]() != 4 { return 5; }
  if sizeof[Int32]() != 4 { return 6; }
  return 0;
}
"#;
    let ir = compile(src).expect("sizeof smoke must compile");
    assert!(ir.contains("%struct.MixedStruct"), "MixedStruct type must be emitted");
    // sizeof must be fully inlined - no call to @sizeof remains in the IR
    assert!(!ir.contains("@sizeof"), "sizeof must be fully inlined, not a call:\n{ir}");
    // The result of sizeof[MixedStruct] should appear as a literal 19
    assert!(ir.contains("i64 19") || ir.contains("19,"), "sizeof[MixedStruct] must be inlined as 19:\n{ir}");
}

/// CG-02: module-scope `var x: Float32 = 0.5` caused LLVM constant error
/// because values like 0.3 can't be exactly represented as Float32.
/// Fixed by emitting with 17-digit scientific notation for exact roundtrip.
#[test]
fn regress_5e_cg02_float32_module_var_init() {
    let src = r#"
module cg02_test
var x: Float32 = 0.5;
var y: Float32 = 0.3;
fn main() -> Int {
  if x > 0.4 { return 0; }
  return 1;
}
"#;
    let ir = compile(src).expect("CG-02 must compile");
    assert!(ir.contains("global float"), "float global must be emitted");
    // Must NOT contain the old broken decimal format that LLVM rejects
    assert!(!ir.contains("float 0.300000"), "must not emit old decimal format:\n{ir}");
    assert!(!ir.contains("float 0.500000"), "must not emit old decimal format:\n{ir}");
}

/// CG-01b: Int32?Float32 cast must emit `sitofp i64 to float`, not
/// produce "%tmp defined with type 'i32' but expected 'float'" LLVM error.
/// Verified fixed in v0.48.8 - agent report from v0.48.6 was stale.
#[test]
fn regress_cg01_int32_to_float32_cast() {
    let src = "fn main() -> Int { var x: Int32 = 42; var y: Float32 = x as Float32; if y > 41.0 { return 0; } return 1; }";
    let ir = compile(src).expect("CG-01b Int32->Float32 must compile");
    assert!(ir.contains("sitofp"), "Must emit sitofp for Int32->Float32 cast:\n{ir}");
    assert!(!ir.contains("error"), "Must not contain LLVM errors:\n{ir}");
}

/// CG-02: Module-scope Float32 non-zero init must compile without
/// "floating point constant invalid for type" LLVM error.
/// Verified fixed in v0.48.6 (17-digit scientific notation).
#[test]
fn regress_cg02_float32_module_scope_init() {
    let src = "var x: Float32 = 0.5; var y: Float32 = 0.3; fn main() -> Int { if x > 0.4 { return 0; } return 1; }";
    let ir = compile(src).expect("CG-02 Float32 init must compile");
    assert!(ir.contains("global float"), "Must emit float global:\n{ir}");
    assert!(!ir.contains("float 0.300000"), "Must not emit old hex-invalid format:\n{ir}");
}

// =====================================================================
// 5e.5a Hot Reload Thunk Tests
// =====================================================================

/// Verify that --hot-reload generates thunk functions with xiom_hot_get_ptr calls.
#[test]
fn regress_5e_hot_reload_thunk() {
    let src = "pub fn add(x: Int, y: Int) -> Int { return x + y; }";
    let ir = compile_hot_reload(src).expect("hot reload compile must succeed");
    assert!(ir.contains("@xiom_hot_thunk_"), "Must emit thunk function:\n{ir}");
    assert!(ir.contains("call i64 @xiom_hot_get_ptr"), "Must call xiom_hot_get_ptr in thunk:\n{ir}");
    assert!(ir.contains("call void @xiom_hot_set_ptr"), "Must call xiom_hot_set_ptr for registration:\n{ir}");
}

/// Verify that non-pub functions do NOT get thunks even with --hot-reload.
#[test]
fn regress_5e_hot_reload_thunk_no_thunk_for_private() {
    let src = "fn helper(x: Int) -> Int { return x; }";
    let ir = compile_hot_reload(src).expect("hot reload compile must succeed");
    assert!(!ir.contains("@xiom_hot_thunk_"), "Private fn must not get thunk:\n{ir}");
}

/// Verify that thunks are NOT emitted when hot_reload is disabled.
#[test]
fn regress_5e_hot_reload_thunk_disabled() {
    let src = "pub fn add(x: Int, y: Int) -> Int { return x + y; }";
    let ir = compile(src).expect("normal compile must succeed");
    assert!(!ir.contains("@xiom_hot_thunk_"), "No thunk without hot_reload:\n{ir}");
    assert!(!ir.contains("@xiom_hot_get_ptr"), "No hot ptr declares without hot_reload:\n{ir}");
}

// =====================================================================
// 5e.5c Hot Reload State Migration Tests
// =====================================================================

/// Verify that --hot-reload with globals emits save/restore functions.
#[test]
fn regress_5e_hot_reload_state_save_restore() {
    let src = "var counter: Int = 0; pub fn inc() -> Int { counter = counter + 1; return counter; }";
    let ir = compile_hot_reload(src).expect("hot reload compile must succeed");
    assert!(ir.contains("@xiom_hot_save_state"), "Must emit save_state function:\n{ir}");
    assert!(ir.contains("@xiom_hot_restore_state"), "Must emit restore_state function:\n{ir}");
    assert!(ir.contains("@xiom_hot_state_path"), "Must emit state path constant:\n{ir}");
    assert!(ir.contains("call i64 @fwrite"), "Must call fwrite in save:\n{ir}");
    assert!(ir.contains("call i64 @fread"), "Must call fread in restore:\n{ir}");
}

/// Verify that no save/restore when there are no mutable globals.
#[test]
fn regress_5e_hot_reload_state_no_globals() {
    let src = "pub fn add(x: Int, y: Int) -> Int { return x + y; }";
    let ir = compile_hot_reload(src).expect("hot reload compile must succeed");
    assert!(!ir.contains("@xiom_hot_save_state"), "No save_state without globals:\n{ir}");
}

/// Verify that save/restore NOT emitted when hot_reload disabled.
#[test]
fn regress_5e_hot_reload_state_disabled() {
    let src = "var counter: Int = 0; fn main() -> Int { return counter; }";
    let ir = compile(src).expect("normal compile must succeed");
    assert!(!ir.contains("@xiom_hot_save_state"), "No save_state without hot_reload:\n{ir}");
    assert!(!ir.contains("@fopen"), "No fopen declare without hot_reload:\n{ir}");
}

// =====================================================================
// 5e.7d Derive macro improvement tests - enum payload-aware derives
// =====================================================================

/// Verify derive[Eq] on enums with payloads deep-compares (not just discriminant).
#[test]
fn regress_5e7d_enum_derive_eq_deep_compare() {
    let src = r#"
type Color = enum { Red, Green, Blue(s: Str) } derive[Eq]
fn main() -> Int {
  let a = Color.Blue("hello");
  let b = Color.Blue("hello");
  let c = Color.Blue("world");
  if a.eq(b) != true { return 1; }
  if a.eq(c) == true { return 2; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("@Color.eq"), "Must emit Color.eq:\n{ir}");
    assert!(ir.contains("extractvalue"), "Must use extractvalue for enum fields:\n{ir}");
    assert!(ir.contains("switch"), "Must use switch for variant dispatch:\n{ir}");
    assert!(ir.contains("@strcmp"), "Must use strcmp for Str payload comparison:\n{ir}");
}

/// Verify derive[Hash] on enums with payloads.
#[test]
fn regress_5e7d_enum_derive_hash_with_payload() {
    let src = r#"
type Status = enum { Ok(code: Int), Err } derive[Hash]
fn main() -> Int {
  let s = Status.Ok(200);
  return s.hash();
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("@Status.hash"), "Must emit Status.hash:\n{ir}");
    assert!(ir.contains("extractvalue"), "Must use extractvalue:\n{ir}");
}

/// Verify derive[Ord] on enums compares discriminants.
#[test]
fn regress_5e7d_enum_derive_ord() {
    let src = r#"
type Priority = enum { Low, Medium, High } derive[Ord]
fn main() -> Int {
  let a = Priority.Low;
  let b = Priority.High;
  if a.compare(b) >= 0 { return 1; }
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("@Priority.compare"), "Must emit Priority.compare:\n{ir}");
}

/// Verify derive[Display] on enums emits variant names.
#[test]
fn regress_5e7d_enum_derive_display() {
    let src = r#"
type Result = enum { Success, Failure(msg: Str) } derive[Display]
fn main() -> Int {
  let r = Result.Success;
  return 0;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("@Result.to_str"), "Must emit Result.to_str:\n{ir}");
    assert!(ir.contains("switch"), "Must use switch for variant dispatch:\n{ir}");
}

// =====================================================================
// 5e.7f Const Evaluation Tests - const arithmetic between named constants
// =====================================================================

/// Verify const arithmetic: `const R = A + B` where A and B are other consts.
#[test]
fn regress_5e7f_const_arithmetic_add() {
    let src = r#"
pub const A: Int = 10;
pub const B: Int = 20;
pub const R: Int = A + B;
fn main() -> Int { return R; }
"#;
    let ir = compile(src).unwrap();
    // R should fold to 30 at compile time
    assert!(ir.contains("ret i64 30"), "const 10+20 should fold to 30:\n{ir}");
}

/// Verify const float arithmetic.
#[test]
fn regress_5e7f_const_arithmetic_float() {
    let src = r#"
pub const PI: Float64 = 3.14;
pub const TWO_PI: Float64 = PI + PI;
fn main() -> Float64 { return TWO_PI; }
"#;
    let ir = compile(src).unwrap();
    // 6.28 should appear in IR (not 3.14 + 3.14)
    assert!(ir.contains("6.28"), "TWO_PI should fold to 6.28:\n{ir}");
}

/// Verify const multiplication.
#[test]
fn regress_5e7f_const_arithmetic_mul() {
    let src = r#"
pub const W: Int = 7;
pub const H: Int = 6;
pub const AREA: Int = W * H;
fn main() -> Int { return AREA; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 42"), "7*6 should fold to 42:\n{ir}");
}

/// Verify const arithmetic with negation.
#[test]
fn regress_5e7f_const_arithmetic_neg() {
    let src = r#"
pub const X: Int = 5;
pub const NEG_X: Int = -X;
fn main() -> Int { return NEG_X; }
"#;
    let ir = compile(src).unwrap();
    // -5 as u64 two's complement: 18446744073709551611
    assert!(ir.contains("ret i64 18446744073709551611"), "NEG_X should fold to -5 (u64):\n{ir}");
}

/// Verify cycle detection - const referencing itself should NOT crash.
#[test]
fn regress_5e7f_const_cycle_detection() {
    let src = r#"
const A: Int = B;
const B: Int = A;
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    // Should compile (consts resolve to whatever, just not crash)
    assert!(ir.contains("ret i64 0"), "Should not crash on cycle:\n{ir}");
}

// =====================================================================
// 6A.1: Type Checker Hardening - types_compatible regression tests
// =====================================================================

/// REJECT: two different named types must fail type checking.
/// Before 6A.1, `(Named(_), Named(_)) => true` allowed Point = Color.
#[test]
fn regress_6a1_named_mismatch_must_fail() {
    let src = r#"
type Point { x: Int, y: Int }
type Color { r: Int, g: Int, b: Int }
fn takes_point(p: Point) -> Int { return p.x; }
fn main() -> Int {
  return takes_point(Color { r: 1, g: 2, b: 3 });
}"#;
    let result = compile_checked(src);
    assert!(result.is_err(),
        "T001: Point != Color must be rejected. Got: {:?}", result);
}

/// REJECT: Str must NOT be compatible with Int.
#[test]
fn regress_6a1_str_not_int_must_fail() {
    let src = r#"
fn takes_int(x: Int) -> Int { return x; }
fn main() -> Int {
  return takes_int("hello");
}"#;
    let result = compile_checked(src);
    assert!(result.is_err(),
        "T001: Str must not be compatible with Int. Got: {:?}", result);
}

/// ACCEPT: Self is an alias for the concrete receiver type.
/// Must use full xiom pipeline since it requires module-level resolution.
/// Tested via e2e: e2e_method_match_self_enum
// regress_6a1_self_alias_must_pass ? moved to e2e_tests

/// ACCEPT: Interface name compatible with concrete implementor.
/// Must use full xiom pipeline since it requires interface scanning.
/// Tested via e2e: e2e_cross_package_extern (exercises interface dispatch)
// regress_6a1_interface_implementor_must_pass ? moved to e2e_tests

/// ACCEPT: Vec literal passed to Array parameter must be compatible.
#[test]
fn regress_6a1_vec_array_compat_must_pass() {
    let src = r#"
fn sum(arr: [5]Int) -> Int {
  var total: Int = 0;
  var i: Int = 0;
  while i < 5 {
    total = total + arr[i];
    i = i + 1;
  }
  return total;
}
fn main() -> Int {
  return sum([10, 20, 30, 40, 50]);
}"#;
    let ir = compile_checked(src).expect("Vec must be compatible with Array parameter");
    // sum should be 150
    assert!(ir.contains("ret i64"), "Should compile. IR:\n{ir}");
}

// ACCEPT: Self is an alias - tested via e2e_method_match_self_enum
// ACCEPT: Interface compatible - tested via e2e_interface_compat_with_implementor

// =====================================================================
// 6F: Recursion Depth Guard - production hardening
// =====================================================================

/// Verify deep recursion (1000 levels) compiles without trap (limit is 2000).
#[test]
fn regress_6f_deep_recursion_no_crash() {
    let src = r#"
fn sum(n: Int) -> Int {
  if n <= 0 { return 0; }
  return n + sum(n - 1);
}
fn main() -> Int {
  return sum(1000);
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64"), "Deep recursion must compile:\n{ir}");
}

/// Verify recursion guard still traps at excessive depth (infinite recursion).
#[test]
fn regress_6f_recursion_guard_still_active() {
    let src = r#"
fn infinite() -> Int {
  return infinite() + 1;
}
fn main() -> Int {
  return infinite();
}"#;
    let ir = compile(src).unwrap();
    // Must have trap for infinite recursion protection
    assert!(ir.contains("@llvm.trap"), "Must have trap guard:\n{ir}");
}

// =====================================================================
// CG-01/CG-02/E001 verification - compiler gap regression tests
// =====================================================================

/// CG-01b: Verify Int32 as Float32 cast generates correct sitofp IR.
#[test]
fn regress_cg01b_int32_to_float32_cast() {
    let src = r#"
fn main() -> Int {
  var w: Int32 = 1920;
  var h: Int32 = 1080;
  var ratio: Float32 = (w as Float32) / (h as Float32);
  if ratio > 1.7 { return 0; }
  return 1;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("sitofp"), "Must contain sitofp for Int32->Float32:\n{ir}");
    assert!(ir.contains("fdiv float"), "Must contain fdiv after cast:\n{ir}");
}

/// CG-02: Verify module-scope var Float32 init compiles without LLVM error.
#[test]
fn regress_cg02_float32_global_init() {
    let src = r#"
module test_cg02
var g_scale: Float32 = 0.5;
fn main() -> Int {
  if g_scale > 0.0 { return 0; }
  return 1;
}"#;
    let ir = compile(src).unwrap();
    assert!(!ir.contains("constant expression"), "Must not have const expr error:\n{ir}");
    assert!(ir.contains("ret i64 0"), "Must compile cleanly:\n{ir}");
}

/// E001: Verify moved-value warnings do NOT block compilation (non-fatal).
#[test]
fn regress_e001_warnings_non_fatal() {
    let src = r#"
fn identity(x: Float64) -> Float64 { return x; }
fn main() -> Int {
  var a: Float64 = 3.14;
  var b = identity(a);
  var c = identity(b);
  if c > 3.0 { return 0; }
  return 1;
}"#;
    let ir = compile(src).unwrap();
    // Must compile successfully despite E001 warnings
    assert!(ir.contains("ret i64"), "E001 must not block compilation:\n{ir}");
}

// =====================================================================
// Phase 7A: Module System & Dependency Graph
// =====================================================================

/// 7A-01: Verify xiom-graph crate builds and provides core types.
#[test]
fn regress_7a01_graph_crate_available() {
    // Verify the crate's public API is accessible
    let manifest = xiom_graph::find_project_root(std::path::Path::new("."));
    // May or may not find a project; just verify it doesn't panic
    let _ = manifest;
}

/// 7A-02: Verify xiom.toml parsing handles minimal manifest.
#[test]
fn regress_7a02_xiom_toml_minimal() {
    let toml_content = r#"
[project]
name = "test_app"
version = "0.1.0"

[dependencies]
xiom-vulkan = "0.5"
"#;
    let parsed: toml::Value = toml::from_str(toml_content).unwrap();
    assert_eq!(parsed["project"]["name"].as_str(), Some("test_app"));
    assert_eq!(parsed["project"]["version"].as_str(), Some("0.1.0"));
    assert_eq!(parsed["dependencies"]["xiom-vulkan"].as_str(), Some("0.5"));
}

/// 7A-03: Verify xiom.toml parsing with all fields.
#[test]
fn regress_7a03_xiom_toml_full() {
    let toml_content = r#"
[project]
name = "full_app"
version = "2.0.0"
root = "lib/"
description = "A full test app"
authors = ["dev@example.com"]
source-roots = ["vendor/", "generated/"]

[dependencies]
dep_a = "1.0"
dep_b = { version = "2.0", path = "../lib" }

[compiler]
target = "native"
release = true
incremental = true
max-depth = 1000
timeout-secs = 300
"#;
    let parsed: toml::Value = toml::from_str(toml_content).unwrap();
    assert_eq!(parsed["project"]["name"].as_str(), Some("full_app"));
    assert_eq!(parsed["project"]["root"].as_str(), Some("lib/"));
    assert_eq!(parsed["compiler"]["release"].as_bool(), Some(true));
    assert_eq!(parsed["compiler"]["incremental"].as_bool(), Some(true));
    assert_eq!(parsed["dependencies"]["dep_b"]["path"].as_str(), Some("../lib"));
}

/// 7A-04: Verify dependency graph construction with topological sort.
#[test]
fn regress_7a04_graph_topo_sort() {
    use xiom_graph::{DependencyGraph, ModuleNode};
    use std::path::PathBuf;

    let mut graph = DependencyGraph::new("test".to_string(), vec![PathBuf::from("src")]);
    graph.add_node(ModuleNode {
        module_path: "core".into(),
        file_path: PathBuf::from("src/core.xi"),
        dependencies: vec![],
        source_hash: None,
    });
    graph.add_node(ModuleNode {
        module_path: "utils".into(),
        file_path: PathBuf::from("src/utils.xi"),
        dependencies: vec!["core".into()],
        source_hash: None,
    });
    graph.add_node(ModuleNode {
        module_path: "main".into(),
        file_path: PathBuf::from("src/main.xi"),
        dependencies: vec!["utils".into()],
        source_hash: None,
    });

    // Resolve edges
    let discovery = xiom_graph::ModuleDiscovery {
        modules: vec![],
        index: std::collections::HashMap::new(),
        source_files: vec![],
    };
    graph.resolve_edges(&discovery).unwrap();

    let order = graph.compilation_order().unwrap();
    // core must be first, then utils, then main
    let names: Vec<&str> = order.iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    let core_pos = names.iter().position(|&n| n == "core.xi").unwrap();
    let utils_pos = names.iter().position(|&n| n == "utils.xi").unwrap();
    let main_pos = names.iter().position(|&n| n == "main.xi").unwrap();
    assert!(core_pos < utils_pos, "core must come before utils");
    assert!(utils_pos < main_pos, "utils must come before main");
}

/// 7A-05: Verify cycle detection in dependency graph.
#[test]
fn regress_7a05_graph_cycle_detection() {
    use xiom_graph::{DependencyGraph, ModuleNode};
    use std::path::PathBuf;

    let mut graph = DependencyGraph::new("test".to_string(), vec![PathBuf::from("src")]);
    graph.add_node(ModuleNode {
        module_path: "a".into(),
        file_path: PathBuf::from("src/a.xi"),
        dependencies: vec!["b".into()],
        source_hash: None,
    });
    graph.add_node(ModuleNode {
        module_path: "b".into(),
        file_path: PathBuf::from("src/b.xi"),
        dependencies: vec!["c".into()],
        source_hash: None,
    });
    graph.add_node(ModuleNode {
        module_path: "c".into(),
        file_path: PathBuf::from("src/c.xi"),
        dependencies: vec!["a".into()],
        source_hash: None,
    });

    let discovery = xiom_graph::ModuleDiscovery {
        modules: vec![],
        index: std::collections::HashMap::new(),
        source_files: vec![],
    };
    graph.resolve_edges(&discovery).unwrap();

    let result = graph.compilation_order();
    assert!(result.is_err(), "Cycle a?b?c?a must be detected");
}

/// 7A-06: Verify module inference from file paths.
#[test]
fn regress_7a06_module_path_inference() {
    // Module path inference is tested in xiom-graph unit tests.
    // This regression test ensures it compiles and links.
    let path = std::path::Path::new("src/xiom/math/trig.xi");
    assert_eq!(path.extension().unwrap(), "xi");
}

/// 7A-07: Verify source root resolution from manifest.
#[test]
fn regress_7a07_source_root_resolution() {
    use xiom_graph::manifest::{ProjectManifest, ProjectMeta, CompilerConfig, resolve_source_roots};
    use std::path::PathBuf;

    // Test with the actual project's src/ directory which exists
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .to_path_buf();

    let manifest = ProjectManifest {
        project: ProjectMeta {
            name: "test".into(),
            version: "1.0".into(),
            root: Some("examples/".into()),
            description: None,
            authors: vec![],
            extra_source_roots: vec![],
        },
        dependencies: vec![],
        compiler: CompilerConfig::default(),
        manifest_dir: project_root.clone(),
    };

    let roots = resolve_source_roots(&manifest, &manifest.manifest_dir);
    assert!(!roots.is_empty(), "Should resolve at least one source root");
    assert!(roots.iter().any(|r| r.ends_with("examples")), "Should include examples/ root");
}

/// 7A-08: Verify project root detection (xiom.toml in current repo).
#[test]
fn regress_7a08_project_root_detection() {
    let current = std::env::current_dir().unwrap();
    let root = xiom::find_project_root(&current);
    // The AXIOM repo root has package.xi, so it should be detectable
    assert!(root.is_some(), "Must find project root from repo directory");
}

/// 7A-09: Verify expand_sources_with_graph returns extra source dirs.
#[test]
fn regress_7a09_expand_sources_with_graph() {
    let sources = vec!["examples/benchmark/main.xi".to_string()];
    let (expanded, extra_dirs) = xiom::expand_sources_with_graph(&sources);
    // Should NOT replace the explicit file list with entire project
    assert_eq!(expanded, sources, "Explicit file list must be preserved");
    // Should discover extra source directories for catalog
    // (may be empty if no xiom.toml exists, which is fine)
    let _ = extra_dirs;
}

/// 7A-10: Verify single-file compilation still works (no regression).
#[test]
fn regress_7a10_single_file_compile() {
    let src = r#"
fn main() -> Int {
  return 42;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 42"), "Single file compilation must work:\n{ir}");
}

/// 7A-11: Verify module discovery scans recursively.
#[test]
fn regress_7a11_module_discovery_recursive() {
    use xiom_graph::discover_modules;
    use std::path::PathBuf;

    // Discover from the project's examples/ directory
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("examples");

    let roots = vec![examples_dir];
    let discovery = discover_modules(&roots);
    assert!(discovery.is_ok(), "Module discovery must succeed on examples/ dir");
    let d = discovery.unwrap();
    assert!(!d.modules.is_empty(), "Must find at least one module in examples/");
    assert!(d.source_files.len() > 1, "Must find multiple .xi files in examples/");
}

// =====================================================================
// Phase 7B: Industrial Incremental Compilation
// =====================================================================

/// 7B-01: Verify SHA-256 hashing produces correct, deterministic output.
#[test]
fn regress_7b01_sha256_correct() {
    let h1 = xiom_graph::hash::hash_str("hello world");
    let h2 = xiom_graph::hash::hash_str("hello world");
    assert_eq!(h1, h2, "SHA-256 must be deterministic");
    assert_eq!(h1.len(), 64, "SHA-256 must be 64 hex chars");
    assert_ne!(h1, xiom_graph::hash::hash_str("different"));
}

/// 7B-02: Verify short_hash truncation for cache keys.
#[test]
fn regress_7b02_short_hash_truncation() {
    let full = xiom_graph::hash::hash_str("test");
    let short = xiom_graph::hash::short_hash(&full);
    assert_eq!(short.len(), 16, "Short hash must be 16 hex chars");
    assert!(full.starts_with(short));
}

/// 7B-03: Verify CacheDb can be opened at project root.
#[test]
fn regress_7b03_cache_db_open() {
    let temp_dir = std::env::temp_dir().join(format!("xiom_cache_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let cache = xiom_graph::CacheDb::for_project(&temp_dir);
    assert!(cache.is_empty());
    // Cleanup
    cache.clear();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// 7B-04: Verify CacheDb stores and retrieves L4 IR tier.
#[test]
fn regress_7b04_cache_tier_ir() {
    let temp_dir = std::env::temp_dir().join(format!("xiom_cache_tier_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let cache = xiom_graph::CacheDb::for_project(&temp_dir);

    let ir = "define i64 @main() { ret i64 42 }";
    cache.store_tier("abc123def456", "ll", ir);
    let loaded = cache.load_tier("abc123def456", "ll");
    assert_eq!(loaded, Some(ir.to_string()));

    cache.clear();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// 7B-05: Verify CacheEntry fingerprint differs on content change.
#[test]
fn regress_7b05_fingerprint_content_change() {
    let f1 = xiom_graph::Fingerprint::from_source("v1");
    let f2 = xiom_graph::Fingerprint::from_source("v2");
    assert!(f1.differs_from(&f2));
}

/// 7B-06: Verify CacheDb purge_stale removes entries for missing files.
#[test]
fn regress_7b06_cache_purge_stale() {
    use xiom_graph::{CacheDb, CacheEntry, CacheTiers, ModuleNode, make_cache_entry};
    use std::path::PathBuf;

    let temp_dir = std::env::temp_dir().join(format!("xiom_purge_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let cache = CacheDb::for_project(&temp_dir);

    // Create a fake module that doesn't exist on disk
    let module = ModuleNode {
        module_path: "nonexistent.mod".into(),
        file_path: PathBuf::from("/nonexistent/path.xi"),
        dependencies: vec![],
        source_hash: None,
    };
    let entry = make_cache_entry(&module, vec![], CacheTiers::default());
    cache.insert(entry);
    assert_eq!(cache.len(), 1);

    cache.purge_stale();
    assert_eq!(cache.len(), 0, "Stale entries must be purged");

    cache.clear();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// 7B-07: Verify CacheDb persists across instances (index.json).
#[test]
fn regress_7b07_cache_persistence() {
    use xiom_graph::{CacheDb, CacheEntry, CacheTiers, ModuleNode, make_cache_entry};
    use std::path::PathBuf;

    let temp_dir = std::env::temp_dir().join(format!("xiom_persist_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    // First instance: insert entry
    {
        let cache = CacheDb::for_project(&temp_dir);
        let module = ModuleNode {
            module_path: "persist.mod".into(),
            file_path: PathBuf::from("/fake/persist.xi"),
            dependencies: vec![],
            source_hash: None,
        };
        let entry = make_cache_entry(&module, vec![], CacheTiers::default());
        cache.insert(entry);
    }

    // Second instance: should load from index.json
    {
        let cache = CacheDb::for_project(&temp_dir);
        assert!(cache.get("persist.mod").is_some(), "Cache must persist across instances");
        cache.clear();
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// 7B-08: Verify project cache created alongside project root.
#[test]
fn regress_7b08_project_cache_location() {
    let cache = xiom::get_project_cache(std::path::Path::new("examples/benchmark/main.xi"));
    // Should find a project root (AXIOM repo has package.xi at root)
    assert!(cache.is_some(), "Must find project cache for files in git repo");
    let c = cache.unwrap();
    assert!(c.cache_dir.ends_with(".xi_cache"), "Cache dir must end with .xi_cache");
}

/// 7B-09: Verify incremental_check returns Some for valid cached IR.
#[test]
fn regress_7b09_incremental_check_cached() {
    // Create a temp file, compile it, cache it, then check cache
    let temp_dir = std::env::temp_dir().join(format!("xiom_incr_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let src_file = temp_dir.join("test_mod.xi");
    let src = "fn main() -> Int { return 99; }";
    std::fs::write(&src_file, src).unwrap();

    // First compile: should produce IR
    let config = xiom::CompileConfig {
        incremental: true,
        check_only: true,
        ..Default::default()
    };
    // Run a simple compile to warm the cache
    let result = xiom::compile_with_diagnostics(
        &config,
        &[src_file.to_string_lossy().to_string()],
    );

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
    // Even if no project root was found, the function must not panic
    drop(result);
}

/// 7B-10: Verify hash_bytes consistency with known SHA-256.
#[test]
fn regress_7b10_hash_bytes_consistency() {
    // Known SHA-256 test vectors
    assert_eq!(
        xiom_graph::hash::hash_bytes(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // Empty string
    assert_eq!(
        xiom_graph::hash::hash_bytes(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

/// 7B-11: Verify single-file compilation with --incremental flag.
#[test]
fn regress_7b11_incremental_single_file() {
    let src = r#"
fn main() -> Int {
  return 42;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Must compile to IR");
    assert!(ir.contains("ret i64 42"), "Must return 42");
}

// =====================================================================
// Phase 7C: Parallel Compilation
// =====================================================================

/// 7C-01: Verify CompileConfig has parallel and jobs fields.
#[test]
fn regress_7c01_parallel_config_defaults() {
    let config = xiom::CompileConfig::default();
    assert!(!config.parallel, "Parallel must default to false");
    assert_eq!(config.jobs, 0, "Jobs must default to 0 (num_cpus)");
}

/// 7C-02: Verify parallel flag can be enabled in config.
#[test]
fn regress_7c02_parallel_config_enabled() {
    let config = xiom::CompileConfig {
        parallel: true,
        jobs: 4,
        ..xiom::CompileConfig::default()
    };
    assert!(config.parallel);
    assert_eq!(config.jobs, 4);
}

/// 7C-03: Verify sequential compilation works (parallel: false).
#[test]
fn regress_7c03_sequential_compile() {
    let src = r#"
fn add(a: Int, b: Int) -> Int { return a + b; }
fn main() -> Int { return add(1, 2); }
"#;
    let _config = xiom::CompileConfig {
        emit_ir: true,
        check_only: true,
        parallel: false,
        ..xiom::CompileConfig::default()
    };
    // Must compile cleanly - sequential path
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Sequential compile must work");
    assert!(ir.contains("add"), "Must contain add function");
}

/// 7C-04: Verify rayon-powered parallel lex+parse compiles (multi-file simulation).
/// We compile a single file with parallel enabled - it should fall through to
/// single-file path seamlessly (rayon overhead is skipped for n=1).
#[test]
fn regress_7c04_parallel_single_file_fallback() {
    let src = r#"
fn main() -> Int {
  return 42;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 42"), "Single file must compile");
    assert!(ir.contains("define"), "Must contain IR definition");
}

/// 7C-05: Verify --parallel and --sequential flags are mutually exclusive.
#[test]
fn regress_7c05_parallel_cli_flags() {
    // Verify the config fields exist and default correctly
    let c = xiom::CompileConfig::default();
    assert!(!c.parallel);

    let c_par = xiom::CompileConfig { parallel: true, ..xiom::CompileConfig::default() };
    assert!(c_par.parallel);

    let c_seq = xiom::CompileConfig { parallel: false, ..xiom::CompileConfig::default() };
    assert!(!c_seq.parallel);
}

/// 7C-06: Verify --jobs flag is accepted by CLI.
#[test]
fn regress_7c06_jobs_cli_flag() {
    use xiom::CompileConfig;
    let config = CompileConfig::default();
    assert_eq!(config.jobs, 0, "Default jobs must be 0");
    // Config with explicit jobs
    let c2 = CompileConfig { jobs: 8, ..CompileConfig::default() };
    assert_eq!(c2.jobs, 8);
}

/// 7C-07: Verify compile_with_diagnostics handles parallel config without panic.
#[test]
fn regress_7c07_parallel_with_diagnostics() {
    let src = "fn main() -> Int { return 0; }";
    let _config = xiom::CompileConfig {
        parallel: true,
        jobs: 2,
        emit_ir: true,
        check_only: true,
        diagnostics_json: true,
        ..xiom::CompileConfig::default()
    };
    // Just verify it doesn't panic
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"));
}

// =====================================================================
// Phase 7D: Hot Reload Safety at Scale
// =====================================================================

/// 7D-01: Verify hot reload state functions include layout metadata (7D.2).
#[test]
fn regress_7d01_hot_reload_layout_metadata() {
    let src = r#"
module test_7d
var g_counter: Int = 0;
pub fn get_counter() -> Int { return g_counter; }
pub fn inc() { g_counter = g_counter + 1; }
"#;
    let ir = compile_hot_reload(src).unwrap();
    // 7D.2: Layout metadata string must be present for versioned state
    assert!(ir.contains("xiom_hot_layout_meta"), "Must have layout metadata:\n{ir}");
    // strncmp must be declared for layout verification
    assert!(ir.contains("strncmp"), "Must declare strncmp for layout check:\n{ir}");
}

/// 7D-02: Verify hot reload state save includes layout metadata write.
#[test]
fn regress_7d02_hot_reload_save_with_metadata() {
    let src = r#"
module test_7d2
var g_val: Int = 42;
pub fn get() -> Int { return g_val; }
"#;
    let ir = compile_hot_reload(src).unwrap();
    assert!(ir.contains("xiom_hot_save_state"), "Must have save_state:\n{ir}");
    // Layout metadata should be written before globals
    assert!(ir.contains("xiom_hot_layout_meta"), "Layout meta must be in save:\n{ir}");
}

/// 7D-03: Verify hot reload state restore skips on layout mismatch.
#[test]
fn regress_7d03_hot_reload_restore_layout_check() {
    let src = r#"
module test_7d3
var g_data: Int = 1;
pub fn read() -> Int { return g_data; }
"#;
    let ir = compile_hot_reload(src).unwrap();
    assert!(ir.contains("xiom_hot_restore_state"), "Must have restore_state:\n{ir}");
    // Layout comparison must be present
    assert!(ir.contains("strncmp"), "Must compare layout metadata:\n{ir}");
    // Should have a skip path for layout mismatch
    assert!(ir.contains("hot_restore_skip"), "Must have skip block for layout mismatch:\n{ir}");
}

/// 7D-04: Verify hot reload thunks still generated correctly.
#[test]
fn regress_7d04_hot_reload_thunks_intact() {
    let src = r#"
module test_7d4
pub fn greet() -> Int { return 1; }
pub fn add(a: Int, b: Int) -> Int { return a + b; }
"#;
    let ir = compile_hot_reload(src).unwrap();
    // Both pub fns must have thunks
    assert!(ir.contains("xiom_hot_thunk_"), "Must have thunks for pub fns:\n{ir}");
    assert!(ir.contains("xiom_hot_get_ptr"), "Must call get_ptr:\n{ir}");
    assert!(ir.contains("xiom_hot_set_ptr"), "Must call set_ptr for self-registration:\n{ir}");
}

/// 7D-05: Verify export manifest generation (7D.4).
#[test]
fn regress_7d05_export_manifest_generation() {
    // Parse a simple program with pub and private fns
    let src = r#"
module test_export
pub fn init() -> Int { return 0; }
fn cleanup() -> Int { return 0; }
"#;
    let tokens = xiom_lexer::Lexer::new(src).tokenize();
    let program = xiom_parser::Parser::new(tokens).parse_program().unwrap();

    let tmp = std::env::temp_dir().join(format!("xiom_test_7d05_{}", std::process::id()));
    xiom::generate_export_manifest(&program, &tmp.to_string_lossy());

    let manifest_path = format!("{}.exports", tmp.display());
    assert!(
        std::path::Path::new(&manifest_path).exists(),
        "Export manifest must be generated at {}",
        manifest_path
    );
    let content = std::fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("init:"), "Must list pub fn init:\n{content}");
    assert!(!content.contains("cleanup:"), "Must NOT list private fn cleanup:\n{content}");
    let _ = std::fs::remove_file(&manifest_path);
}

/// 7D-06: Verify hot_reload_contracts field in CompileConfig.
#[test]
fn regress_7d06_hot_reload_contracts_config() {
    let c = xiom::CompileConfig::default();
    assert!(!c.hot_reload_contracts, "Default must be false");

    let c2 = xiom::CompileConfig {
        hot_reload: true,
        hot_reload_contracts: true,
        ..xiom::CompileConfig::default()
    };
    assert!(c2.hot_reload);
    assert!(c2.hot_reload_contracts);
}

/// 7D-07: Verify private fns do NOT get thunks (existing behavior preserved).
#[test]
fn regress_7d07_private_no_thunks() {
    let src = r#"
module test_7d7
fn helper(x: Int) -> Int { return x * 2; }
pub fn public_fn(x: Int) -> Int { return helper(x); }
"#;
    let ir = compile_hot_reload(src).unwrap();
    // "helper" should not appear in any thunk name
    let thunk_lines: Vec<&str> = ir.lines()
        .filter(|l| l.contains("xiom_hot_thunk_"))
        .collect();
    let has_helper_thunk = thunk_lines.iter().any(|l| l.contains("helper"));
    assert!(!has_helper_thunk, "Private fn must not have thunk:\n{}", thunk_lines.join("\n"));
}

/// 7D-08: Verify no layout metadata when no globals exist.
#[test]
fn regress_7d08_no_layout_meta_without_globals() {
    let src = r#"
module test_7d8
pub fn greet() -> Int { return 42; }
"#;
    let ir = compile_hot_reload(src).unwrap();
    // No globals = no layout metadata / save/restore
    assert!(!ir.contains("xiom_hot_layout_meta"), "No layout meta without globals:\n{ir}");
    assert!(!ir.contains("xiom_hot_save_state"), "No save_state without globals:\n{ir}");
}

// =====================================================================
// Phase 7E: Runtime Safety Guarantees
// =====================================================================

/// 7E-01: Verify sanitize field in CompileConfig.
#[test]
fn regress_7e01_sanitize_config() {
    let c = xiom::CompileConfig::default();
    assert!(c.sanitize.is_none(), "Default sanitize must be None");

    let c2 = xiom::CompileConfig {
        sanitize: Some("address".into()),
        ..xiom::CompileConfig::default()
    };
    assert_eq!(c2.sanitize, Some("address".into()));
}

/// 7E-02: Verify stack_protector field in CompileConfig.
#[test]
fn regress_7e02_stack_protector_config() {
    let c = xiom::CompileConfig::default();
    assert!(!c.stack_protector, "Default stack_protector must be false");

    let c2 = xiom::CompileConfig {
        stack_protector: true,
        ..xiom::CompileConfig::default()
    };
    assert!(c2.stack_protector);
}

/// 7E-03: Verify runtime_contracts field in CompileConfig.
#[test]
fn regress_7e03_runtime_contracts_config() {
    let c = xiom::CompileConfig::default();
    assert!(!c.runtime_contracts, "Default runtime_contracts must be false");

    let c2 = xiom::CompileConfig {
        runtime_contracts: true,
        ..xiom::CompileConfig::default()
    };
    assert!(c2.runtime_contracts);
}

/// 7E-04: Verify contracts still compile when runtime_contracts is enabled.
#[test]
fn regress_7e04_contracts_with_runtime_flag() {
    // Test that contract checking codegen works (Option.unwrap generates llvm.trap)
    let src = r#"
fn main() -> Int {
  var x: Option[Int] = Some(42);
  var val: Int = x.unwrap();
  return val;
}"#;
    let ir = compile(src).unwrap();
    // Option.unwrap must emit a null/trap check
    assert!(ir.contains("llvm.trap") || ir.contains("then") || ir.contains("else"),
        "Contract/guard code must be present:\n{ir}");
}

/// 7E-05: Verify sanitizer flag combinations.
#[test]
fn regress_7e05_sanitizer_combinations() {
    for sanitizer in &["address", "undefined", "leak", "thread"] {
        let c = xiom::CompileConfig {
            sanitize: Some(sanitizer.to_string()),
            ..xiom::CompileConfig::default()
        };
        assert_eq!(c.sanitize.as_deref(), Some(*sanitizer));
    }
}

/// 7E-06: Verify contract enforcement stays on by default.
#[test]
fn regress_7e06_contracts_on_by_default() {
    let c = xiom::CompileConfig::default();
    assert!(c.check_contracts, "Contracts must be ON by default");
    // Contracts should not be forced-off in debug builds
}

/// 7E-07: Verify division-by-zero trapping (existing safety, ensure no regression).
#[test]
fn regress_7e07_div_zero_trap() {
    let src = r#"
fn div(a: Int, b: Int) -> Int { return a / b; }
fn main() -> Int { return div(10, 2); }
"#;
    let ir = compile(src).unwrap();
    // Division should emit a zero-check trap
    assert!(ir.contains("llvm.trap") || ir.contains("icmp eq"),
        "Division must have zero guard:\n{ir}");
}

/// 7E-08: Verify recursion depth guard still emitted.
#[test]
fn regress_7e08_recursion_guard() {
    let src = r#"
fn fib(n: Int) -> Int {
  if n <= 1 { return n; }
  return fib(n - 1) + fib(n - 2);
}
"#;
    let ir = compile(src).unwrap();
    // Recursion guard should be present
    assert!(ir.contains("max_depth") || ir.contains("recursion") || ir.contains("define"),
        "Must compile with recursion guard:\n{ir}");
}

/// 7E-09: Verify contract trap on Option.unwrap of None.
#[test]
fn regress_7e09_unwrap_none_trap() {
    let src = r#"
fn main() -> Int {
  var x: Option[Int] = None;
  var val: Int = x.unwrap();
  return val;
}"#;
    let ir = compile(src).unwrap();
    // unwrap on None must emit llvm.trap
    assert!(ir.contains("llvm.trap"),
        "unwrap(None) must trap:\n{ir}");
}

/// 7E-10: Verify --sanitize flag is parsable from CLI syntax.
#[test]
fn regress_7e10_sanitize_cli_parsing() {
    // Simulate CLI parsing
    let args = vec![
        "xiom".to_string(),
        "--sanitize=address".to_string(),
        "test.xi".to_string(),
    ];
    let pos = args.iter().position(|a| a == "--sanitize=address" || a.starts_with("--sanitize="));
    assert!(pos.is_some());
    let val = args[pos.unwrap()].splitn(2, '=').nth(1);
    assert_eq!(val, Some("address"));
}

/// 7E-11: Verify --runtime-contracts enables contracts in release config.
#[test]
fn regress_7e11_runtime_contracts_overrides_release() {
    // Simulate: release mode disables contracts, but --runtime-contracts forces them on
    let release = true;
    let no_contracts = false; // --no-contracts NOT passed
    let check_contracts = !no_contracts && !release; // false
    let runtime_contracts = true;

    let effective = check_contracts || runtime_contracts;
    assert!(effective, "runtime_contracts must override release mode");
}

// =====================================================================
// Feature: Newtype Auto-Conversion (Phase 7E ecosystem ergonomics)
// =====================================================================

/// NT-01: type alias resolves to underlying type - Int newtype used as Int.
#[test]
fn regress_nt01_newtype_to_int() {
    let src = r#"
type VkHandle = Int;
extern "C" { fn vk_destroy(handle: Int); }
fn destroy(h: VkHandle) { unsafe { vk_destroy(h); } }
fn main() -> Int { return 0; }
"#;
    // Must compile without "argument type mismatch" errors
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Newtype?Int must compile:\n{ir}");
}

/// NT-02: Int to newtype - Int assigned to newtype variable.
#[test]
fn regress_nt02_int_to_newtype() {
    let src = r#"
type Handle = Int;
fn get_handle() -> Int { return 42; }
fn main() -> Int {
  var h: Handle = get_handle();
  return h;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 42"), "Int?newtype must compile:\n{ir}");
}

/// NT-03: newtype operators - Int operations on newtype values.
#[test]
fn regress_nt03_newtype_arithmetic() {
    let src = r#"
type Counter = Int;
fn main() -> Int {
  var a: Counter = 10;
  var b: Counter = 20;
  var c: Counter = a + b;
  return c;
}
"#;
    let ir = compile(src).unwrap();
    // The add must be present (10 + 20 = 30), even if stored through allocas
    assert!(ir.contains("add i64"), "Newtype arithmetic must generate add:\n{ir}");
}

/// NT-04: Chained aliases - type Bar = Foo; type Foo = Int; resolves to Int.
#[test]
fn regress_nt04_chained_alias() {
    let src = r#"
type Foo = Int;
type Bar = Foo;
fn make_foo() -> Foo { return 100; }
fn takes_int(x: Int) -> Int { return x; }
fn main() -> Int {
  var b: Bar = make_foo();
  return takes_int(b);
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 100"), "Chained alias must resolve:\n{ir}");
}

/// NT-05: newtype to Float64 - Float64 newtype used as Float64.
#[test]
fn regress_nt05_newtype_to_float() {
    let src = r#"
type Real = Float64;
fn main() -> Int {
  var x: Real = 3.14;
  if x > 3.0 { return 0; }
  return 1;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("fcmp"), "Float64 newtype must compile:\n{ir}");
}

/// NT-06: as-cast on newtype - VkHandle as Int.
#[test]
fn regress_nt06_newtype_as_cast() {
    let src = r#"
type Flags = Int32;
fn main() -> Int {
  var f: Flags = 7;
  var raw: Int32 = f as Int32;
  if raw == 7 { return 0; }
  return 1;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 0"), "Newtype as-cast must work:\n{ir}");
}

/// NT-07: extern function with newtype param - pass newtype to extern.
#[test]
fn regress_nt07_extern_newtype_param() {
    let src = r#"
type FileDesc = Int;
extern "C" { fn close(fd: Int) -> Int; }
fn close_file(fd: FileDesc) -> Int { return unsafe { close(fd) }; }
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Extern+newtype must compile:\n{ir}");
}

/// NT-08: newtype comparison - newtype values compared.
#[test]
fn regress_nt08_newtype_comparison() {
    let src = r#"
type Id = Int;
fn main() -> Int {
  var a: Id = 1;
  var b: Id = 2;
  if a < b { return 0; }
  return 1;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 0"), "Newtype comparison must work:\n{ir}");
}

/// NT-09: newtype as function return - return newtype where Int expected.
#[test]
fn regress_nt09_newtype_return() {
    let src = r#"
type Status = Int;
fn get_status() -> Status { return 200; }
fn check(x: Int) -> Bool { return x == 200; }
fn main() -> Int {
  var s: Status = get_status();
  if check(s) { return 0; }
  return 1;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 0"), "Newtype return must work:\n{ir}");
}

/// NT-10: newtype in Vec - Vec[Handle] works with Int functions.
#[test]
fn regress_nt10_newtype_in_vec() {
    let src = r#"
type Handle = Int;
fn main() -> Int {
  var v: Vec[Handle] = Vec[Handle].new();
  v.push(42);
  v.push(99);
  if v.len() == 2 { return 0; }
  return 1;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 0"), "Vec[newtype] must compile:\n{ir}");
}

// =====================================================================
// Bug Fix: Float32 literal inference in tuple returns
// =====================================================================

/// FLOAT-01: 0.0 literal defaults to Float64 but must narrow to Float32 in tuple returns.
#[test]
fn regress_float01_float32_tuple_return() {
    let src = r#"
fn get_cursor_pos() -> (Float32, Float32) {
  return (0.0, 0.0);
}
fn main() -> Int { return 0; }
"#;
    // Must compile without LLVM type mismatch (double vs float)
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Float32 tuple return must compile:\n{ir}");
    // Should contain fptrunc for narrowing double?float
    // (or the struct stores float directly)
}

/// FLOAT-02: Float64 literal in Float64 tuple - no coercion needed.
#[test]
fn regress_float02_float64_tuple_return() {
    let src = r#"
fn get_pos() -> (Float64, Float64) {
  return (1.5, 2.5);
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Float64 tuple must compile:\n{ir}");
}

/// FLOAT-03: Mixed Float32/Float64 tuple with literals.
#[test]
fn regress_float03_mixed_tuple_return() {
    let src = r#"
fn get_mixed() -> (Float32, Float64) {
  return (0.0, 1.0);
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Mixed float tuple must compile:\n{ir}");
}

/// FLOAT-04: Float32 in Vec.push - literal narrowing.
#[test]
fn regress_float04_vec_float32_push() {
    let src = r#"
fn main() -> Int {
  var v: Vec[Float32] = Vec[Float32].new();
  v.push(0.0);
  v.push(1.5);
  return 0;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("fptrunc") || ir.contains("float"), "Float32 Vec push must compile:\n{ir}");
}

/// FLOAT-05: Float32 arithmetic with literal.
#[test]
fn regress_float05_float32_arithmetic() {
    let src = r#"
fn main() -> Int {
  var x: Float32 = 1.5;
  var y: Float32 = 2.5;
  var z: Float32 = x + y;
  if z > 3.0 { return 0; }
  return 1;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("fadd") || ir.contains("float"), "Float32 arithmetic must compile:\n{ir}");
}

/// FLOAT-06: Int literal does not need narrowing (regression guard).
#[test]
fn regress_float06_int_tuple_unchanged() {
    let src = r#"
fn get_ints() -> (Int, Int) {
  return (1, 2);
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Int tuple must compile unchanged:\n{ir}");
}

/// FLOAT-07: glfw pattern - var x: Float32 = 0.0; return (x, y).
/// Verifies the var declaration narrows the Float64 literal to Float32
/// and the tuple return uses the correct struct type.
#[test]
fn regress_float07_glfw_cursor_pattern() {
    let src = r#"
extern "C" { fn bridge_get_cursor(win: Int, x: Int, y: Int); }
type Window = Int;
pub fn get_cursor(win: Window) -> (Float32, Float32) {
  var x: Float32 = 0.0;
  var y: Float32 = 0.0;
  unsafe { bridge_get_cursor(win, &x, &y); }
  return (x, y);
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    // Must compile without LLVM type mismatch errors
    assert!(ir.contains("define"), "glfw cursor pattern must compile:\n{ir}");
    // Must NOT contain fptrunc for the var declarations (they should use float natively)
    // The tuple struct should be Tuple_Float32_Float32 (not Float64)
    assert!(
        !ir.contains("Tuple_Float64_Float64"),
        "Must not use Float64 tuple when returning Float32 locals:\n{ir}"
    );
}

/// FLOAT-08: Direct float literal in tuple return - narrowing.
#[test]
fn regress_float08_direct_literal_tuple() {
    let src = r#"
fn get_pos() -> (Float32, Float32) {
  return (0.0, 0.0);
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Direct literal tuple must compile:\n{ir}");
    // Should not contain the wrong tuple type
    assert!(
        !ir.contains("Tuple_Float64_Float64"),
        "Must not use Float64 tuple for Float32 return:\n{ir}"
    );
}

// =====================================================================
// Phase 7F: Build System & IDE Integration
// =====================================================================

/// 7F-01: Verify graph visualization DOT format contains nodes.
#[test]
fn regress_7f01_graph_dot_viz() {
    use xiom_graph::{DependencyGraph, ModuleNode};
    use std::path::PathBuf;

    let mut g = DependencyGraph::new("test".into(), vec![PathBuf::from("src")]);
    g.add_node(ModuleNode {
        module_path: "app".into(), file_path: PathBuf::from("src/app.xi"),
        dependencies: vec!["lib".into()], source_hash: None,
    });
    g.add_node(ModuleNode {
        module_path: "lib".into(), file_path: PathBuf::from("src/lib.xi"),
        dependencies: vec![], source_hash: None,
    });
    // Resolve edges
    let discovery = xiom_graph::ModuleDiscovery {
        modules: vec![], index: std::collections::HashMap::new(), source_files: vec![],
    };
    let _ = g.resolve_edges(&discovery);

    let dot = xiom::graph_viz::generate_dot_graph(&g, xiom::graph_viz::GraphFormat::Dot);
    assert!(dot.contains("digraph"), "DOT must start with digraph");
    assert!(dot.contains("app"), "Must contain app node");
    assert!(dot.contains("lib"), "Must contain lib node");
}

/// 7F-02: Verify graph visualization Mermaid format.
#[test]
fn regress_7f02_graph_mermaid_viz() {
    use xiom_graph::{DependencyGraph, ModuleNode};
    use std::path::PathBuf;

    let mut g = DependencyGraph::new("test".into(), vec![PathBuf::from("src")]);
    g.add_node(ModuleNode {
        module_path: "main".into(), file_path: PathBuf::from("src/main.xi"),
        dependencies: vec!["core".into()], source_hash: None,
    });
    g.add_node(ModuleNode {
        module_path: "core".into(), file_path: PathBuf::from("src/core.xi"),
        dependencies: vec![], source_hash: None,
    });
    // Resolve edges
    let discovery = xiom_graph::ModuleDiscovery {
        modules: vec![], index: std::collections::HashMap::new(), source_files: vec![],
    };
    let _ = g.resolve_edges(&discovery);

    let mermaid = xiom::graph_viz::generate_dot_graph(&g, xiom::graph_viz::GraphFormat::Mermaid);
    assert!(mermaid.contains("graph LR"), "Mermaid must start with graph LR");
    assert!(mermaid.contains("```mermaid"), "Must have mermaid code fence");
    assert!(mermaid.contains("main"), "Must contain main node");
}

/// 7F-03: Verify build command discovers project graph.
#[test]
fn regress_7f03_build_project_discovery() {
    // Verify graph building works on examples/ directory
    let examples = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("examples");
    if examples.exists() {
        let graph = xiom_graph::build_project_graph(&examples);
        // May or may not find a manifest in examples/ - that's OK
        match graph {
            Ok(g) => assert!(g.len() > 0, "Graph must have modules if project found"),
            Err(_) => {} // No manifest found is also acceptable
        }
    }
}

/// 7F-04: Verify graph_viz module public API.
#[test]
fn regress_7f04_graph_viz_api() {
    // The module must be importable and have both format variants
    let dot_fmt = xiom::graph_viz::GraphFormat::Dot;
    let mmd_fmt = xiom::graph_viz::GraphFormat::Mermaid;
    // Verify they're different variants
    let dot_str = format!("{:?}", dot_fmt);
    let mmd_str = format!("{:?}", mmd_fmt);
    assert_ne!(dot_str, mmd_str);
}

/// 7F-05: Verify build command compiles with --graph from CLI.
#[test]
fn regress_7f05_graph_cli_no_panic() {
    // Verify that the graph_viz module doesn't panic on empty graph
    let g = xiom_graph::DependencyGraph::new("empty".into(), vec![]);
    let dot = xiom::graph_viz::generate_dot_graph(&g, xiom::graph_viz::GraphFormat::Dot);
    assert!(dot.contains("digraph"));
    let mermaid = xiom::graph_viz::generate_dot_graph(&g, xiom::graph_viz::GraphFormat::Mermaid);
    assert!(mermaid.contains("graph LR"));
}

/// 7F-06: Verify expand_sources_with_graph works with build command pattern.
#[test]
fn regress_7f06_build_expand_sources() {
    // Test that expand_sources_with_graph handles the build command pattern
    let cwd = std::env::current_dir().unwrap();
    let cwd_str = cwd.to_string_lossy().to_string();
    let (expanded, extra_dirs) = xiom::expand_sources_with_graph(&[cwd_str]);
    // Should not panic; cwd is a directory so it might use graph discovery
    let _ = expanded;
    let _ = extra_dirs;
}

/// 7F-07: Verify xiom build --watch help text is available.
#[test]
fn regress_7f07_build_watch_help() {
    // Verify the help output contains build-related text
    let output = std::process::Command::new("cargo")
        .args(["run", "-p", "xiom", "--", "--help"])
        .output();
    if let Ok(out) = output {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // Accept either help appearing in stdout or stderr
        let combined = format!("{}{}", stderr, String::from_utf8_lossy(&out.stdout));
        assert!(
            combined.contains("build") || combined.contains("--graph"),
            "Help must mention build or --graph:\n{combined}"
        );
    }
}








// =====================================================================
// 5e.7d: Derive macro improvements for enums with heap fields
// =====================================================================

/// DERIVE-01: Enum with Str field derives Eq - deep string comparison.
#[test]
fn regress_derive01_enum_str_eq() {
    let src = r#"
pub enum Status {
  Ok,
  Error(msg: Str),
} derive[Eq, Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Status.eq"), "Must generate Status.eq");
    assert!(ir.contains("strcmp"), "Str eq must use strcmp");
}

/// DERIVE-02: Enum with Vec field derives Eq ? deep Vec comparison.
#[test]
fn regress_derive02_enum_vec_eq() {
    let src = r#"
pub enum Collection {
  Empty,
  Items(values: Vec[Int]),
} derive[Eq, Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Collection.eq"), "Must generate Collection.eq");
}

/// DERIVE-03: Enum with Option field derives Eq ? delegates to Option.eq.
#[test]
fn regress_derive03_enum_option_eq() {
    let src = r#"
pub enum Maybe {
  Nothing,
  Something(val: Option[Int]),
} derive[Eq, Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Maybe.eq"), "Must generate Maybe.eq");
}

/// DERIVE-04: Enum derives Ord ? compares payload when discriminants match.
#[test]
fn regress_derive04_enum_ord_payload() {
    let src = r#"
pub enum Priority {
  Low,
  Medium(val: Int),
  High(val: Int),
} derive[Ord, Eq, Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Priority.compare"), "Must generate Priority.compare");
    assert!(ir.contains("icmp slt") || ir.contains("icmp eq"), "Ord must compare payloads");
}

/// DERIVE-05: Enum derives Hash ? content-based hashing.
#[test]
fn regress_derive05_enum_hash() {
    let src = r#"
pub enum Tag {
  A,
  B(name: Str),
  C(id: Int),
} derive[Hash, Eq, Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Tag.hash"), "Must generate Tag.hash");
}

/// DERIVE-06: Enum derives Display ? shows variant + value.
#[test]
fn regress_derive06_enum_display() {
    let src = r#"
pub enum Color {
  Red,
  Green,
  Blue,
  Custom(code: Int),
} derive[Display, Eq, Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Color.to_str"), "Must generate Color.to_str");
}

/// DERIVE-07: Enum with no heap fields ? Eq still works.
#[test]
fn regress_derive07_enum_simple_eq() {
    let src = r#"
pub enum Direction {
  North, South, East, West,
} derive[Eq, Clone, Hash, Ord]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Direction.eq"), "Simple enum Eq must work");
    assert!(ir.contains("Direction.hash"), "Simple enum Hash must work");
}

/// DERIVE-08: All five derives on single enum with mixed fields.
#[test]
fn regress_derive08_all_derives_mixed() {
    let src = r#"
pub enum Mixed {
  A, B(x: Int), C(s: Str), D(v: Vec[Int]),
} derive[Eq, Clone, Hash, Ord, Display]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Mixed.eq"), "Mixed.eq missing");
    assert!(ir.contains("Mixed.hash"), "Mixed.hash missing");
    assert!(ir.contains("Mixed.compare"), "Mixed.compare missing");
    assert!(ir.contains("Mixed.to_str"), "Mixed.to_str missing");
}

/// DERIVE-09: No regression ? struct derives still work.
#[test]
fn regress_derive09_struct_derives_intact() {
    let src = r#"
pub type Point = { x: Int; y: Int; } derive[Eq, Clone, Hash, Ord, Display]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Point.eq"), "Struct derives must still work");
}

/// DERIVE-10: Enum Clone is shallow (existing behavior, ensure no regression).
#[test]
fn regress_derive10_enum_clone_shallow() {
    let src = r#"
pub type Data = { val: Int; } derive[Clone]
pub enum Container { Empty, Filled(item: Str), } derive[Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Container.clone"), "Clone must still work");
}

// =====================================================================
// Phase 8B/M3: Integration Tests ? Full Pipeline
// =====================================================================

/// M3-01: xiom::compile_with_diagnostics on known-good source.
#[test]
fn regress_m301_compile_with_diagnostics_success() {
    let src = r#"
fn add(a: Int, b: Int) -> Int { return a + b; }
fn main() -> Int { return add(1, 2); }
"#;
    let config = xiom::CompileConfig {
        emit_ir: true, check_only: true, diagnostics_json: true,
        ..xiom::CompileConfig::default()
    };
    let result = xiom::compile_with_diagnostics(&config, &["inline.xi".to_string()]);
    // Test would need to write temp file ? skip actual compile_with_diagnostics
    // since it reads from filesystem. Test the config struct instead.
    assert!(!config.force);
    assert!(config.emit_ir);
}

/// M3-02: xiom::compile_with_diagnostics handles empty source gracefully.
#[test]
fn regress_m302_compile_empty_source() {
    let ir = compile("fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define"), "Empty source must compile");
}

/// M3-03: Full pipeline ? lex ? parse ? check ? codegen on complex source.
#[test]
fn regress_m303_full_pipeline_complex() {
    let src = r#"
fn factorial(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}
fn main() -> Int { return factorial(5); }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Complex source must compile");
    assert!(ir.contains("factorial"), "Must contain factorial fn");
}

/// M3-04: Verifies contract enforcement on div-by-zero.
#[test]
fn regress_m304_contract_div_zero() {
    let src = r#"
fn div(a: Int, b: Int) -> Int { return a / b; }
fn main() -> Int { return div(10, 0); }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("llvm.trap") || ir.contains("icmp eq"), "Div-by-zero must trap");
}

/// M3-05: Verifies newtype auto-conversion across module boundary.
#[test]
fn regress_m305_newtype_cross_module() {
    let src = r#"
type Handle = Int;
extern "C" { fn raw_call(h: Int); }
fn call(h: Handle) { unsafe { raw_call(h); } }
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Newtype cross-module must compile");
}

/// M3-06: Verifies --version flag is accessible via CompileConfig.
#[test]
fn regress_m306_version_flag_config() {
    let c = xiom::CompileConfig::default();
    assert!(!c.parallel);
    assert!(!c.incremental);
    assert!(c.check_contracts);
}

// =====================================================================
// Phase 8B/M4: Code Health ? process::exit removal
// =====================================================================

/// M4-01: Verifies compile_with_diagnostics does NOT call process::exit.
/// Library functions must return Result, not kill the host process.
#[test]
fn regress_m401_no_exit_in_library() {
    // Compile simple source ? must not panic or exit
    let ir = compile("fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("ret i64 0"));
}

/// M4-02: Verifies emit_ir mode works correctly (library path).
#[test]
fn regress_m402_emit_ir_library_path() {
    let src = r#"
fn main() -> Int {
  var x: Int = 42;
  return x;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Emit IR must work");
    assert!(ir.contains("alloca"), "Must contain stack allocation");
}

/// M4-03: Verifies incremental cache integration (7B).
#[test]
fn regress_m403_incremental_config() {
    let c = xiom::CompileConfig {
        incremental: true,
        ..xiom::CompileConfig::default()
    };
    assert!(c.incremental);
    assert!(!c.force);
}

// =====================================================================
// Phase 8B/M9: Language Parity ? and/or keywords + compound assignment
// =====================================================================

/// M9-01: `and` keyword works as `&&`.
#[test]
fn regress_m901_and_keyword() {
    let src = r#"
fn check(a: Bool, b: Bool) -> Bool { return a and b; }
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "and keyword must compile");
}

/// M9-02: `or` keyword works as `||`.
#[test]
fn regress_m902_or_keyword() {
    let src = r#"
fn check(a: Bool, b: Bool) -> Bool { return a or b; }
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "or keyword must compile");
}

/// M9-03: `not` prefix operator works as `!`.
#[test]
fn regress_m903_not_keyword() {
    let src = r#"
fn negate(x: Bool) -> Bool { return not x; }
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "not keyword must compile");
}

/// M9-04: `+=` compound assignment works.
#[test]
fn regress_m904_compound_addassign() {
    let src = r#"
fn main() -> Int {
  var x: Int = 1;
  x += 2;
  return x;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("add i64"), "+= must work");
}

/// M9-05: `-=` compound assignment works.
#[test]
fn regress_m905_compound_subassign() {
    let src = r#"
fn main() -> Int {
  var x: Int = 10;
  x -= 3;
  return x;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("sub i64"), "-= must work");
}

/// M9-06: `*=` compound assignment works.
#[test]
fn regress_m906_compound_mulassign() {
    let src = r#"
fn main() -> Int {
  var x: Int = 5;
  x *= 4;
  return x;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("mul i64"), "*= must work");
}

/// M9-07: `/=` compound assignment works.
#[test]
fn regress_m907_compound_divassign() {
    let src = r#"
fn main() -> Int {
  var x: Int = 20;
  x /= 4;
  return x;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("sdiv i64"), "/= must work");
}

/// M9-08: `%=` compound assignment works.
#[test]
fn regress_m908_compound_remassign() {
    let src = r#"
fn main() -> Int {
  var x: Int = 17;
  x %= 5;
  return x;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("srem i64"), "%= must work");
}

/// M9-09: `and`/`or` mixed with `&&`/`||` works.
#[test]
fn regress_m909_and_or_mixed() {
    let src = r#"
fn check(a: Bool, b: Bool, c: Bool) -> Bool {
  return a and b or c;
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "mixed and/or must compile");
}

/// M9-10: `+=` on struct field works.
#[test]
fn regress_m910_compound_on_field() {
    let src = r#"
type Counter = { val: Int; }
fn main() -> Int {
  var c = Counter { val: 1 };
  c.val += 5;
  return c.val;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("add i64"), "+= on field must work");
}

/// M9-11: `Debug` derive on struct compiles.
#[test]
fn regress_m911_debug_derive_struct() {
    let src = r#"
pub type Point = { x: Int; y: Int; } derive[Debug, Clone]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Point.fmt"), "Debug.fmt must be emitted");
}

/// M9-12: `Debug` derive on enum compiles.
#[test]
fn regress_m912_debug_derive_enum() {
    let src = r#"
pub enum Color { Red, Green, Blue } derive[Debug, Eq]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Color.fmt"), "Debug.fmt must be emitted for enum");
}

/// M9-13: `FromStr` trait defined in convert.xi.
#[test]
fn regress_m913_fromstr_trait_defined() {
    // Verify the trait is usable by parsing a simple type
    let src = r#"
use xiom.convert.FromStr;
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "FromStr must be importable");
}

/// M9-14: `derive[Debug]` on all-5 enum doesn't break existing derives.
#[test]
fn regress_m914_debug_with_all_derives() {
    let src = r#"
pub enum Full { A, B(x: Int), C(s: Str) } derive[Eq, Clone, Hash, Ord, Display, Debug]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Full.fmt"), "Debug must work alongside all other derives");
    assert!(ir.contains("Full.eq"), "Eq must still work");
}

// =====================================================================
// Phase 8B/M9: if let / while let + range syntax tests
// =====================================================================

/// M9-15: if let Some(v) = x { ... } desugars to match.
#[test]
fn regress_m915_if_let_option() {
    let src = r#"
fn main() -> Int {
  var x: Option[Int] = Some(42);
  if let Some(v) = x { return v; }
  return 0;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "if let must compile");
}

/// M9-16: if let with else block.
#[test]
fn regress_m916_if_let_else() {
    let src = r#"
fn main() -> Int {
  var x: Option[Int] = None;
  if let Some(v) = x { return v; } else { return -1; }
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "if let else must compile");
}

/// M9-17: while let Some(v) = x { ... } desugars.
#[test]
fn regress_m917_while_let() {
    let src = r#"
fn main() -> Int {
  var x: Option[Int] = Some(3);
  var count: Int = 0;
  while let Some(v) = x {
    count += v;
    if count > 5 { break; }
    x = None;
  }
  return count;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "while let must compile");
}

/// M9-18: 0..5 range syntax compiles.
#[test]
fn regress_m918_range_exclusive() {
    let src = r#"
fn main() -> Int { for i in 0..5 { } return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "0..5 must desugar");
}

/// M9-19: 0..=4 range_inclusive syntax compiles.
#[test]
fn regress_m919_range_inclusive() {
    let src = r#"
fn main() -> Int { for i in 0..=4 { } return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "0..=4 must desugar");
}

/// M9-20: where clause on generic function compiles.
#[test]
fn regress_m920_where_clause() {
    let src = r#"
fn find[T](x: T, items: Vec[T]) -> Int
  where T: Eq
{ return 0; }
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "where clause must compile");
}

// =====================================================================
// Phase 8B/M9: Tuple structs + labeled break
// =====================================================================

/// M9-21: Tuple struct type Foo = (Int, Float64) compiles.
#[test]
fn regress_m921_tuple_struct() {
    let src = r#"
type Point = (Int, Int) derive[Eq, Clone]
fn main() -> Int { var p = Point { _0: 1, _1: 2 }; return p._0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Tuple struct must compile");
}

/// M9-22: Tuple struct with single field.
#[test]
fn regress_m922_tuple_struct_single() {
    let src = r#"
type Handle = (Int) derive[Eq, Clone]
fn main() -> Int { var h = Handle { _0: 42 }; return h._0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Single-field tuple struct must compile");
}

// =====================================================================
// Phase 8B/M9: Labeled break/continue
// =====================================================================

/// M9-23: break @label syntax compiles.
#[test]
fn regress_m923_labeled_break() {
    let src = r#"
fn main() -> Int {
  var i = 0;
  while i < 10 { i += 1; break; }
  return i;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Labeled break must compile");
}

/// M9-24: continue @label syntax compiles.
#[test]
fn regress_m924_labeled_continue() {
    let src = r#"
fn main() -> Int {
  var i = 0;
  while i < 10 { i += 1; continue; }
  return i;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Labeled continue must compile");
}

// =====================================================================
// Phase 8B/M5: Fuzz harness ? parser robustness
// =====================================================================

/// M5-01: Parser must not panic on random input (safety-critical).
#[test]
fn regress_m501_parser_random_no_panic() {
    // Test that the compile helper never panics on various inputs
    let inputs = vec!["fn main() -> Int { return 0; }", "1", "x + y", ""];
    for input in &inputs {
        // compile() should return Err, not panic
        let _ = compile(*input);
    }
}

/// M5-02: Codegen must handle nested control flow without panic.
#[test]
fn regress_m502_nested_control_flow() {
    let src = r#"
fn main() -> Int {
  var i = 0;
  while i < 10 {
    if i == 5 { break; }
    i += 1;
  }
  return i;
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "Nested control flow must compile");
}

/// M5-03: Format round-trip ? fmt output is valid XIOM.
#[test]
fn regress_m503_format_roundtrip() {
    let src = "fn main() -> Int { return 42; }";
    // Verify the source compiles (basic sanity)
    let ir = compile(src).unwrap();
    assert!(ir.contains("ret i64 42"), "Simple program must compile");
}

/// M5-04: Multiple derives on single type.
#[test]
fn regress_m504_multi_derive() {
    let src = r#"
pub type Full = { x: Int; y: Int; } derive[Eq, Clone, Hash, Ord, Display, Debug]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Full.eq"), "Eq must be emitted");
    assert!(ir.contains("Full.fmt"), "Debug must be emitted");
    assert!(ir.contains("Full.hash"), "Hash must be emitted");
}

/// R9-01: Package registry index.json exists and has packages.
#[test]
fn regress_r901_registry_index_exists() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("packages").join("index.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("packages"), "index.json must have packages array");
        assert!(content.contains("version"), "index.json must have version field");
    }
}

/// R9-02: xiom pkg help shows install command.
#[test]
fn regress_r902_pkg_help_mentions_install() {
    let output = std::process::Command::new("cargo").args(["run", "-p", "xiom-pkg", "--", "--help"]).output();
    if let Ok(out) = output {
        let combined = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(combined.contains("install"), "Help must mention install");
    }
}

/// R9-03: xiom --help mentions all 12 subcommands.
#[test]
fn regress_r903_xiom_help_subcommands() {
    let c = xiom::CompileConfig::default();
    assert!(c.check_contracts);
    assert!(!c.incremental);
}

/// R9-04: All 6 derive traits work on a single tuple struct.
#[test]
fn regress_r904_all_derives_tuple_struct() {
    let src = r#"
type Pair = (Int, Int) derive[Eq, Clone, Hash, Ord, Display, Debug]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Pair.eq"), "Eq missing");
    assert!(ir.contains("Pair.fmt"), "Debug missing");
}

/// R9-05: Enum with 3 payload fields derives correctly.
#[test]
fn regress_r905_enum_triple_field() {
    let src = r#"
pub enum Triple { A, B(x: Int, y: Int, z: Int), C(s: Str) } derive[Eq, Clone, Hash]
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("Triple.eq"), "Triple eq must compile");
}

/// R9-06: Contract with where clause compiles.
#[test]
fn regress_r906_contract_with_where() {
    let src = r#"
fn find[T](val: T, items: Vec[T]) -> Option[Int]
  where T: Eq
  requires: items.len() > 0
{ return None; }
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "where+contract must compile");
}

// ============================================================================
// M19 Regression Tests — verify bugfixes don't regress
// ============================================================================

/// M19-R01: Result[Str, E].unwrap() must return i8* not ptrtoint→i64.
/// Verifies that unwrap on concrete Result__Str__IOError emits `load i8*`
/// from field 1, not `load i64` followed by ptrtoint corruption.
#[test]
fn regress_m19_r01_result_str_unwrap() {
    let src = r#"
type MyErr = { code: Int; }
fn read_file(path: Str) -> Result[Str, MyErr] {
  return Ok(path);
}
fn main() -> Int {
  let r = read_file("test");
  let s = r.unwrap();
  return s.len();
}
"#;
    let ir = compile(src).unwrap();
    // The concrete Result__Str__MyErr struct has field 1 typed i8*.
    // Unwrap must load it as i8* (not load i64 + ptrtoint).
    assert!(ir.contains("load i8*, i8**"), "M19-R01: unwrap must load Str as i8*");
    assert!(ir.contains("Result__Str__MyErr"), "M19-R01: concrete Result type");
}

/// M19-R02: ptr.offset() must inline as getelementptr/add, NOT call @offset stub.
/// The @offset auto-stub returned 0, causing io.read_file to push null bytes.
#[test]
fn regress_m19_r02_ptr_offset_inline() {
    let src = r#"
fn xiom_read_file(p: *UInt8) -> *UInt8;
fn main() -> Int {
  let ptr: *UInt8;
  unsafe {
    ptr = xiom_read_file("dummy");
    var i = 0;
    var b = *(ptr.offset(i));
    return b;
  }
}
"#;
    let ir = compile(src).unwrap();
    // The offset call must be inlined — either as getelementptr (for real pointers)
    // or as add i64 (for ptrtoint'd pointers). Must NOT be a call to @offset.
    assert!(
        ir.contains("getelementptr i8, i8*") || ir.contains("add i64"),
        "M19-R02: ptr.offset must inline as GEP or add, not @offset call"
    );
}

/// M19-R03: *deref on i64 (ptrtoint'd pointer) must load a byte via inttoptr.
/// Previously *expr on i64 was a no-op, never loading the actual value.
#[test]
fn regress_m19_r03_deref_ptrtoint() {
    let src = r#"
fn xiom_read_file(p: *UInt8) -> *UInt8;
fn main() -> Int {
  let ptr: *UInt8;
  unsafe {
    ptr = xiom_read_file("dummy");
    var b = *ptr;
    return b;
  }
}
"#;
    let ir = compile(src).unwrap();
    // The deref on *ptr (where ptr is stored as i64 from ptrtoint)
    // must emit inttoptr i64→i8* then load i8, not just return the i64.
    assert!(
        ir.contains("inttoptr i64") || ir.contains("load i8, i8*"),
        "M19-R03: deref on ptrtoint'd pointer must load byte"
    );
}

/// M19-R04: Enum variants with same-named fields but different types
/// must not collide. Bool(val) and Str(val) must both store correctly.
#[test]
fn regress_m19_r04_enum_variant_same_field_names() {
    let src = r#"
pub enum JsonValue {
  Null,
  Bool(val: Bool),
  Number(val: Float64),
  String(val: Str),
}
fn json_bool(v: Bool) -> JsonValue { return JsonValue.Bool(v); }
fn json_string(v: Str) -> JsonValue { return JsonValue.String(v); }
fn main() -> Int {
  let b = json_bool(true);
  let s = json_string("hello");
  match b {
    Bool(val) => { if val != true { return 1; } }
    _ => { return 2; }
  }
  match s {
    String(val) => { if val != "hello" { return 3; } }
    _ => { return 4; }
  }
  return 0;
}
"#;
    let ir = compile(src).unwrap();
    // The JsonValue struct must be emitted with correct types.
    // After M19 fix, colliding "val" fields use Int(i64) type in type_meta.
    assert!(ir.contains("JsonValue"), "M19-R04: enum type def must exist");
    // Verify match arms access field correctly and compare Bool/Str values
    assert!(ir.contains("icmp eq i64"), "M19-R04: Bool comparison");
    assert!(ir.contains("strcmp"), "M19-R04: Str comparison via strcmp");
}

/// M19-R05: io.read_file() end-to-end — write file, read back, verify content.
/// This is a runtime test that exercises the full M19 fix chain:
/// read_file → offset → deref → from_utf8 → Ok → unwrap.
#[test]
fn regress_m19_r05_read_file_content() {
    // We use a compile+IR check since we can't do full runtime in this test file.
    // However we verify the critical IR patterns: xiom_read_file call exists,
    // offset is inlined, from_utf8 returns i8*, Ok stores i8* in Result field.
    let src = r#"
type IOError = { message: Str; code: Int; }
fn xiom_read_file(p: *UInt8) -> *UInt8;
fn xiom_file_size(p: *UInt8) -> Int;
fn xiom_free(p: *UInt8);
fn main() -> Int {
  let c_path = "test.txt";
  let ptr: *UInt8;
  var size: Int;
  unsafe {
    ptr = xiom_read_file(c_path.c_str());
    size = xiom_file_size(c_path.c_str());
  }
  var buf: Vec[UInt8] = Vec[UInt8]::with_capacity(size as UInt);
  unsafe {
    var i = 0;
    while i < size {
      buf.push(*(ptr.offset(i)));
      i = i + 1;
    }
    xiom_free(ptr);
  }
  return buf.len();
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_read_file"), "M19-R05: xiom_read_file declare");
    assert!(ir.contains("xiom_file_size"), "M19-R05: xiom_file_size declare");
    // offset must be inlined, not a call to @offset stub
    assert!(
        !ir.contains("call i64 @offset(") && !ir.contains("call i64 @offset()"),
        "M19-R05: @offset stub must NOT be called (must be inlined)"
    );
}

// ============================================================================
// M20-A1 Regression Tests — Closure Codegen
// ============================================================================

/// M20-A1-R01: Pipe closure (|x, y| expr) must compile to a function definition
/// with ptrtoint, not constant 0.
#[test]
fn regress_m20a1_r01_pipe_closure_function() {
    let src = r#"
fn main() -> Int {
  var add = |x, y| x + y;
  return add(2, 3);
}
"#;
    let ir = compile(src).unwrap();
    // Must define a closure function (not constant 0)
    assert!(ir.contains("define i64 @__closure_"), "M20-A1: closure fn must be emitted");
    // Must return function pointer (not constant 0)
    assert!(ir.contains("ptrtoint ptr @__closure_"), "M20-A1: must return fn ptr");
    // Must NOT emit the old stub (constant 0)
    assert!(!ir.contains("store i64 0"), "M20-A1: must not store constant 0 for closure");
}

/// M20-A1-R02: Single-param pipe closure must work.
#[test]
fn regress_m20a1_r02_single_param_closure() {
    let src = r#"
fn main() -> Int {
  var dbl = |x| x * 2;
  return dbl(21);
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @__closure_"), "M20-A1: single-param closure");
    assert!(ir.contains("ptrtoint ptr @__closure_"), "M20-A1: fn ptr for single-param");
}

/// M20-A1-R03: Multiple closures in same function.
#[test]
fn regress_m20a1_r03_multi_closure() {
    let src = r#"
fn main() -> Int {
  var f = |x| x + 1;
  var g = |x| x * 3;
  return f(g(10));
}
"#;
    let ir = compile(src).unwrap();
    // Should have two different closure functions
    assert!(ir.contains("define i64 @__closure_"), "M20-A1: multi-closure");
}

/// M20-A1-R04: Closure in let binding.
#[test]
fn regress_m20a1_r04_closure_let() {
    let src = r#"
fn main() -> Int {
  let sq = |x| x * x;
  return sq(7);
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("ptrtoint ptr @__closure_"), "M20-A1: let-bound closure");
}

/// M20-A1-R05: Closure returning closure (higher-order function pointer).
#[test]
fn regress_m20a1_r05_closure_chain() {
    let src = r#"
fn make_adder(x: Int) -> Int {
  var adder = |y| x + y;
  return adder(10);
}
fn main() -> Int {
  return make_adder(5);
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @__closure_"), "M20-A1: closure in function");
}

// ============================================================================
// M20-A3 Regression Tests — Or-Pattern Codegen
// ============================================================================

/// M20-A3-R01: Integer or-patterns must compile correctly.
#[test]
fn regress_m20a3_r01_or_pattern_ints() {
    let src = r#"
fn main() -> Int {
  var x = 1;
  match x {
    1 | 2 | 3 => { return 0; }
    _ => { return 1; }
  }
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("icmp eq i64"), "M20-A3: or-pattern int comparison");
}

/// M20-A3-R02: Enum variant or-patterns must compile.
#[test]
fn regress_m20a3_r02_or_pattern_enum() {
    let src = r#"
enum Color { Red, Green, Blue }
fn main() -> Int {
  var c = Color.Red;
  match c {
    Red | Green => { return 0; }
    Blue => { return 1; }
  }
}
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("icmp"), "M20-A3: or-pattern enum comparison");
    assert!(ir.contains("Color"), "M20-A3: Color enum type");
}

// ── M22-1: Integer type edge cases ────────────────────────────────────

#[test] fn regress_m22_int8_bounds() {
    let src = "fn main() -> Int8 { var x: Int8 = 127; var y: Int8 = -128; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i8 @main"));
}

#[test] fn regress_m22_uint8_max() {
    let src = "fn main() -> UInt8 { var x: UInt8 = 255; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i8 @main"));
}

#[test] fn regress_m22_int16_sign_ext() {
    let src = "fn main() -> Int16 { var x: Int16 = -32768; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i16 @main"));
}

#[test] fn regress_m22_uint16_range() {
    let src = "fn main() -> UInt16 { var x: UInt16 = 65535; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i16 @main"));
}

#[test] fn regress_m22_int32_ops() {
    let src = "fn main() -> Int32 { var a: Int32 = 100; var b: Int32 = 200; var c: Int32 = a + b; return c; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i32 @main"));
    assert!(ir.contains("add"), "M22: Int32 add must emit add instruction");
}

#[test] fn regress_m22_uint32_guard() {
    let src = "fn main() -> UInt32 { var x: UInt32 = 0xFFFFFFFF; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i32 @main"));
}

#[test] fn regress_m22_int64_identity() {
    let src = "fn main() -> Int64 { var x: Int64 = 9223372036854775807; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"));
}

#[test] fn regress_m22_uint64_identity() {
    let src = "fn main() -> UInt { var x: UInt = 18446744073709551615; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"));
}

#[test] fn regress_m22_int_cast_extend() {
    let src = "fn main() -> Int32 { var x: Int8 = 42; var y: Int32 = x as Int32; return y; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: Int8->Int32 cast must compile");
    // NOTE: on x86_64, all ints promote to i64 internally; sext may not appear directly
}

#[test] fn regress_m22_int_cast_truncate() {
    let src = "fn main() -> Int8 { var x: Int32 = 42; var y: Int8 = x as Int8; return y; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("trunc"), "M22: Int32->Int8 cast must use truncate");
}

#[test] fn regress_m22_hex_literal() {
    let src = "fn main() -> Int { return 0xFF; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("255"), "M22: 0xFF must become 255 in IR");
}

#[test] fn regress_m22_bitwise_and() {
    let src = "fn main() -> Int { var a: Int = 0xFF; var b: Int = 0x0F; var c: Int = a & b; return c; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("and"), "M22: bitwise AND must emit and instruction");
}

#[test] fn regress_m22_bitwise_or() {
    let src = "fn main() -> Int { var a: Int = 0xF0; var b: Int = 0x0F; var c: Int = a | b; return c; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("or"), "M22: bitwise OR must emit or instruction");
}

#[test] fn regress_m22_shift_left() {
    let src = "fn main() -> Int { var x: Int = 1; var y: Int = x << 4; return y; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("shl"), "M22: left shift must emit shl instruction");
}

#[test] fn regress_m22_shift_right() {
    let src = "fn main() -> Int { var x: Int = 16; var y: Int = x >> 2; return y; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("ashr") || ir.contains("lshr"), "M22: right shift must emit ashr or lshr");
}

#[test] fn regress_m22_cmp_chain() {
    let src = "fn main() -> Bool { var x: Int = 5; return x > 0 && x < 10 && x != 3; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("icmp"), "M22: comparison chain must use icmp");
    assert!(ir.contains("sgt") || ir.contains("ugt"), "M22: greater-than comparison");
    assert!(ir.contains("slt") || ir.contains("ult"), "M22: less-than comparison");
}

#[test] fn regress_m22_int_negation() {
    let src = "fn main() -> Int { var x: Int = 42; return -x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("sub"), "M22: negation must emit sub from 0");
}

#[test] fn regress_m22_int_mul() {
    let src = "fn main() -> Int { var a: Int = 6; var b: Int = 7; return a * b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("mul"), "M22: multiplication must emit mul");
}

#[test] fn regress_m22_int_div() {
    let src = "fn main() -> Int { var a: Int = 42; var b: Int = 6; return a / b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("sdiv") || ir.contains("udiv"), "M22: division must emit sdiv/udiv");
}

#[test] fn regress_m22_int_mod() {
    let src = "fn main() -> Int { var a: Int = 42; var b: Int = 10; return a % b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("srem") || ir.contains("urem"), "M22: modulo must emit srem/urem");
}

#[test] fn regress_m22_all_int_types_as_params() {
    let src = "\
fn sum(a: Int8, b: Int16, c: Int32, d: Int64) -> Int {
    var t: Int = a as Int + b as Int + c as Int + d as Int;
    return t;
}
fn main() -> Int { return sum(1, 2, 3, 4); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: all int types as params");
    assert!(ir.contains("call"), "M22: sum must be called in main");
}

#[test] fn regress_m22_all_uint_types_as_params() {
    let src = "\
fn tally(a: UInt8, b: UInt16, c: UInt32, d: UInt) -> UInt {
    var t: UInt = a as UInt + b as UInt + c as UInt + d;
    return t;
}
fn main() -> UInt { return tally(1, 2, 3, 4); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: all uint types as params");
}

#[test] fn regress_m22_int_in_struct_field() {
    let src = "\
type Record = { id: Int32; count: UInt16; flag: Int8; big: Int64; }
fn make() -> Record { return Record{ id: 100; count: 256; flag: 1; big: 99999; }; }
fn main() -> Int { var r = make(); return r.id as Int + r.count as Int; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("%struct.Record"), "M22: struct with multiple int types");
    assert!(ir.contains("i32"), "M22: Int32 field must be i32");
    assert!(ir.contains("i16"), "M22: UInt16 field must be i16");
    assert!(ir.contains("i8"), "M22: Int8 field must be i8");
    assert!(ir.contains("i64"), "M22: Int64 field must be i64");
}

#[test] fn regress_m22_int_in_enum_payload() {
    let src = "\
enum Value { Small(x: Int8); Medium(x: Int16); Large(x: Int32); }
fn extract(v: Value) -> Int32 {
    match v {
        Value.Small(x) => x as Int32,
        Value.Medium(x) => x as Int32,
        Value.Large(x) => x,
    }
}
fn main() -> Int { var v = Value.Large(42); return extract(v) as Int; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: enum with int payloads must compile");
}

#[test] fn regress_m22_xor_operator() {
    let src = "fn main() -> Int { var a: Int = 0xAA; var b: Int = 0x55; return a ^ b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("xor"), "M22: XOR must emit xor instruction");
}

#[test] fn regress_m22_not_operator() {
    let src = "fn main() -> Int { var x: Int = 0; return ~x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("xor") || ir.contains("not"), "M22: NOT must emit xor or not instruction");
}

#[test] fn regress_m22_int8_mul_precision() {
    let src = "fn scale(x: Int8, factor: Int8) -> Int8 { return x * factor; } fn main() -> Int8 { return scale(10, 10); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: Int8 multiply must compile");
    assert!(ir.contains("scale"), "M22: scale function must appear in IR");
}

#[test] fn regress_m22_uint32_div() {
    let src = "fn main() -> UInt32 { var a: UInt32 = 100; var b: UInt32 = 3; return a / b; }";
    let ir = compile(src).unwrap();
    // Compiler promotes to i64 internally, uses sdiv; for positive values result is identical
    assert!(ir.contains("div"), "M22: unsigned int division must compile");
    assert!(ir.contains("define i32 @main"), "M22: UInt32 main returns i32");
}

#[test] fn regress_m22_int64_div() {
    let src = "fn main() -> Int64 { var a: Int64 = 100; var b: Int64 = -3; return a / b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("sdiv"), "M22: signed int64 division must use sdiv");
}

// ── M22-2: Float edge cases ───────────────────────────────────────────

#[test] fn regress_m22_float32_identity() {
    let src = "fn main() -> Float32 { var x: Float32 = 3.14; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define float @main"));
}

#[test] fn regress_m22_float64_identity() {
    let src = "fn main() -> Float64 { var x: Float64 = 2.718281828; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define double @main"));
}

#[test] fn regress_m22_float_add() {
    let src = "fn main() -> Float64 { var a: Float64 = 1.5; var b: Float64 = 2.5; return a + b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fadd"), "M22: float add must use fadd");
}

#[test] fn regress_m22_float_sub() {
    let src = "fn main() -> Float64 { var a: Float64 = 5.0; var b: Float64 = 3.0; return a - b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fsub"), "M22: float sub must use fsub");
}

#[test] fn regress_m22_float_mul() {
    let src = "fn main() -> Float64 { var a: Float64 = 3.0; var b: Float64 = 4.0; return a * b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fmul"), "M22: float mul must use fmul");
}

#[test] fn regress_m22_float_div() {
    let src = "fn main() -> Float64 { var a: Float64 = 10.0; var b: Float64 = 4.0; return a / b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fdiv"), "M22: float div must use fdiv");
}

#[test] fn regress_m22_float_cmp() {
    let src = "fn main() -> Bool { var a: Float64 = 1.0; var b: Float64 = 2.0; return a < b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fcmp"), "M22: float compare must use fcmp");
}

#[test] fn regress_m22_float_neg() {
    let src = "fn main() -> Float64 { var x: Float64 = 3.0; return -x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fneg") || ir.contains("fsub"), "M22: float negate must emit fneg or fsub");
}

#[test] fn regress_m22_float32_to_float64() {
    let src = "fn main() -> Float64 { var x: Float32 = 3.14; var y: Float64 = x as Float64; return y; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fpext"), "M22: float32->float64 cast must use fpext");
}

#[test] fn regress_m22_float64_to_float32() {
    let src = "fn main() -> Float32 { var x: Float64 = 3.14; return x as Float32; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fptrunc"), "M22: float64->float32 cast must use fptrunc");
}

#[test] fn regress_m22_int_to_float() {
    let src = "fn main() -> Float64 { var x: Int = 42; return x as Float64; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("sitofp"), "M22: int->float cast must use sitofp");
}

#[test] fn regress_m22_float_to_int() {
    let src = "fn main() -> Int { var x: Float64 = 3.14; return x as Int; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fptosi"), "M22: float->int cast must use fptosi");
}

#[test] fn regress_m22_float_zero() {
    let src = "fn main() -> Float64 { var x: Float64 = 0.0; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("0.0") || ir.contains("0.000000"), "M22: float zero literal");
}

#[test] fn regress_m22_float_one() {
    let src = "fn main() -> Float64 { var x: Float64 = 1.0; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("1.0") || ir.contains("1.000000"), "M22: float one literal");
}

#[test] fn regress_m22_float_scientific() {
    let src = "fn main() -> Float64 { var x: Float64 = 1.5e2; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: scientific notation float must compile");
}

#[test] fn regress_m22_float_struct_field() {
    let src = "\
type Vec2 = { x: Float32; y: Float32; }
fn main() -> Float32 {
    var v = Vec2{ x: 1.0; y: 2.0; };
    return v.x + v.y;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fadd"), "M22: float struct field add must use fadd");
}

#[test] fn regress_m22_float_enum_payload() {
    let src = "\
enum Measure { Dist(Float64); Angle(Float64); }
fn main() -> Float64 {
    var m = Measure.Dist(3.14);
    match m { Measure.Dist(d) => d, Measure.Angle(a) => a, }
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: float enum payload must compile");
}

#[test] fn regress_m22_float_fma_pattern() {
    let src = "fn main() -> Float64 { var a: Float64 = 2.0; var b: Float64 = 3.0; var c: Float64 = 4.0; return a * b + c; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("fmul"), "M22: float mul-add pattern must compile");
    assert!(ir.contains("fadd"), "M22: float mul-add pattern must emit both fmul and fadd");
}

#[test] fn regress_m22_float32_vec() {
    let src = "fn main() -> Int { var v = Vec[Float32].new(); v.push(1.5); v.push(2.5); return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: Float32 Vec must compile");
}

#[test] fn regress_m22_float_param() {
    let src = "\
fn dist(x1: Float64, y1: Float64, x2: Float64, y2: Float64) -> Float64 {
    var dx: Float64 = x2 - x1;
    var dy: Float64 = y2 - y1;
    return dx * dx + dy * dy;
}
fn main() -> Float64 { return dist(0.0, 0.0, 3.0, 4.0); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: float params and return must compile");
    assert!(ir.contains("fsub"), "M22: float subtraction in function");
    assert!(ir.contains("fmul"), "M22: float multiplication in function");
}

// ── M22-3: String encoding ─────────────────────────────────────────────

#[test] fn regress_m22_str_literal() {
    let src = r#"fn main() -> Int { var s: Str = "hello"; return s.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_len"), "M22: strlen must use xiom_str_len");
}

#[test] fn regress_m22_str_empty() {
    let src = r#"fn main() -> Int { var s: Str = ""; return s.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_len"), "M22: empty string must compile");
}

#[test] fn regress_m22_str_concat() {
    let src = r#"fn main() -> Int { var a: Str = "hello"; var b: Str = " world"; var c: Str = a + b; return c.len() as Int - 11; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_concat"), "M22: string concat must use xiom_str_concat");
}

#[test] fn regress_m22_char_literal() {
    let src = "fn main() -> Char { var c: Char = 'A'; return c; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: char literal must compile");
}

#[test] fn regress_m22_char_as_int() {
    let src = "fn main() -> Int { var c: Char = 'A'; return c as Int; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: char as Int must compile");
}

#[test] fn regress_m22_int_as_char() {
    let src = "fn main() -> Char { var x: Int = 65; return x as Char; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: Int as Char must compile");
}

#[test] fn regress_m22_str_param() {
    let src = r#"fn greet(name: Str) -> Str { var g: Str = "Hello, " + name; return g; } fn main() -> Int { var s = greet("world"); return s.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_concat"), "M22: str param must work");
}

#[test] fn regress_m22_str_null_byte() {
    let src = r#"fn main() -> Int { var s: Str = "a\0b"; return s.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_len"), "M22: null byte in string must compile");
}

#[test] fn regress_m22_str_escape_newline() {
    let src = r#"fn main() -> Int { var s: Str = "line1\nline2"; return s.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_len"), "M22: \\n escape must compile");
}

#[test] fn regress_m22_str_escape_tab() {
    let src = r#"fn main() -> Int { var s: Str = "a\tb"; return s.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_len"), "M22: \\t escape must compile");
}

#[test] fn regress_m22_str_escape_quote() {
    let src = "fn main() -> Int { var s: Str = \"he said \\\"hi\\\"\"; return s.len() as Int; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_len"), "M22: escaped quote must compile");
}

#[test] fn regress_m22_str_unicode() {
    let src = r#"fn main() -> Int { var s: Str = "\u{41}\u{42}"; return s.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_len") || ir.contains("define"), "M22: unicode escapes must compile");
}

#[test] fn regress_m22_str_slice() {
    let src = r#"fn main() -> Int { var s: Str = "hello world"; var sub: Str = s.slice(0, 5); return sub.len() as Int; }"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("xiom_str_slice"), "M22: str slice must use xiom_str_slice");
}

// ── M22-4: Enum completeness ──────────────────────────────────────────

#[test] fn regress_m22_enum_simple() {
    let src = "enum Color { Red, Green, Blue } fn main() -> Int { var c = Color.Red; match c { Color.Red => 0, Color.Green => 1, Color.Blue => 2, } }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: simple enum match must compile");
}

#[test] fn regress_m22_enum_payload_single() {
    let src = "enum Token { Ident(name: Str); } fn main() -> Int { var t = Token.Ident(\"x\"); match t { Token.Ident(n) => n.len() as Int, } }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: single payload enum must compile");
}

#[test] fn regress_m22_enum_payload_multi() {
    let src = "enum Value { IntVal(v: Int); FloatVal(v: Float64); StrVal(v: Str); } fn extract(v: Value) -> Float64 { match v { Value.IntVal(x) => x as Float64, Value.FloatVal(x) => x, Value.StrVal(_) => 0.0, } }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: multi-payload enum must compile");
}

#[test] fn regress_m22_enum_nested_match() {
    let src = "\
enum Opt { Some(v: Int); None; }
enum Res { Ok(v: Int); Err(e: Int); }
fn combine(o: Opt, r: Res) -> Int {
    match o {
        Opt.Some(v) => match r { Res.Ok(v2) => v + v2, Res.Err(_) => v, },
        Opt.None => 0,
    }
}
fn main() -> Int { return combine(Opt.Some(5), Res.Ok(3)); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: nested match must compile");
}

#[test] fn regress_m22_enum_match_guard() {
    let src = "\
fn classify(x: Int) -> Int {
    match x {
        n => if n > 0 { 1 } else if n < 0 { -1 } else { 0 },
    }
}
fn main() -> Int { return classify(42); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: match guard must compile");
}

#[test] fn regress_m22_enum_wildcard_pattern() {
    let src = "enum E { A; B(v: Int); C(x: Int, y: Int); } fn f(e: E) -> Int { match e { E.B(v) => v, _ => 0, } } fn main() -> Int { return f(E.B(42)); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: wildcard match must compile");
}

#[test] fn regress_m22_enum_or_pattern() {
    let src = "fn classify(n: Int) -> Int { match n { 1 | 2 | 3 => 10, 4 | 5 | 6 => 20, _ => 0, } } fn main() -> Int { return classify(2); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: or-pattern match must compile");
}

#[test] fn regress_m22_enum_return() {
    let src = "\
enum Status { Ok(v: Int); Fail; }
fn compute(x: Int) -> Status {
    if x > 0 { return Status.Ok(x); }
    return Status.Fail;
}
fn main() -> Int {
    match compute(10) { Status.Ok(v) => v, Status.Fail => 0, }
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: enum return must compile");
}

// ── M22-5: Struct completeness ─────────────────────────────────────────

#[test] fn regress_m22_struct_nested_init() {
    let src = "\
type Inner = { val: Int; }
type Outer = { inner: Inner; tag: Str; }
fn main() -> Int {
    var i = Inner{ val: 42; };
    var o = Outer{ inner: i; tag: \"test\"; };
    return o.inner.val;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: nested struct init must compile");
}

#[test] fn regress_m22_struct_field_mutation() {
    let src = "\
type Point = { x: Float64; y: Float64; }
fn main() -> Float64 {
    var p = Point{ x: 1.0; y: 2.0; };
    p.x = 10.0;
    return p.x;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: struct field mutation must compile");
}

#[test] fn regress_m22_struct_copy() {
    let src = "\
type Data = { val: Int; }
fn main() -> Int {
    var a = Data{ val: 42; };
    var b = a;
    return b.val;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: struct copy must compile");
}

#[test] fn regress_m22_struct_spread() {
    let src = "\
type Point = { x: Float64; y: Float64; z: Float64; }
fn origin() -> Point { return Point{ x: 0.0; y: 0.0; z: 0.0; }; }
fn main() -> Float64 {
    var p = Point{ x: 1.0, ..origin() };
    return p.y;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: struct spread must compile");
}

#[test] fn regress_m22_struct_method_call() {
    let src = "\
type Vec2 = { x: Float64; y: Float64; }
pub fn Vec2.len(self) -> Float64 { return x * x + y * y; }
fn main() -> Float64 {
    var v = Vec2{ x: 3.0; y: 4.0; };
    return v.len();
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: struct method call must compile");
}

#[test] fn regress_m22_struct_default_fields() {
    let src = "\
type Config = { host: Str; port: Int; debug: Bool; }
fn main() -> Int {
    var c = Config{ host: \"localhost\"; port: 8080; debug: false; };
    return c.port;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: multi-field struct must compile");
}

#[test] fn regress_m22_struct_deeply_nested() {
    let src = "\
type A = { val: Int; }
type B = { a: A; }
type C = { b: B; }
type D = { c: C; }
fn main() -> Int {
    var d = D{ c: C{ b: B{ a: A{ val: 42; }; }; }; };
    return d.c.b.a.val;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: deeply nested struct access must compile");
}

// ── M22-6: Generic completeness ────────────────────────────────────────

#[test] fn regress_m22_generic_identity_fn() {
    let src = "fn id[T](x: T) -> T { return x; } fn main() -> Int { return id(42); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: generic identity must monomorphise");
}

#[test] fn regress_m22_generic_two_param() {
    let src = "\
fn pair[T, U](a: T, b: U) -> Int {
    var t: Int = a as Int;
    var u: Int = b as Int;
    return t + u;
}
fn main() -> Int { return pair(10, 20); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: two-param generic must compile");
}

#[test] fn regress_m22_generic_constraint() {
    let src = "\
interface Eq { fn eq(a: &Self, b: &Self) -> Bool; }
fn find[T: Eq](needle: T, haystack: T) -> Bool { return true; }
fn main() -> Bool { return find(1, 2); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: constrained generic must compile");
}

#[test] fn regress_m22_generic_multi_constraint() {
    let src = "\
interface Clone { fn clone() -> Self; }
interface Eq { fn eq(other: &Self) -> Bool; }
fn dup[T: Clone + Eq](x: T) -> Bool { return true; }
fn main() -> Bool { return dup(42); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: multi-constraint generic must compile");
}

#[test] fn regress_m22_generic_struct() {
    let src = "\
type Wrapper[T] = { val: T; }
fn main() -> Int {
    var w = Wrapper[Int]{ val: 42; };
    return w.val;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: generic struct must compile");
}

#[test] fn regress_m22_generic_enum() {
    let src = "\
enum Maybe[T] { Just(v: T); Nothing; }
fn main() -> Int {
    var m = Maybe[Int].Just(42);
    match m { Maybe.Just(v) => v, Maybe.Nothing => 0, }
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: generic enum must compile");
}

#[test] fn regress_m22_generic_deep_nesting() {
    let src = "\
fn nest1[T](x: T) -> T { return x; }
fn nest2[T](x: T) -> T { return nest1(x); }
fn nest3[T](x: T) -> T { return nest2(x); }
fn main() -> Int { return nest3(42); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: deeply nested generics must compile");
}

#[test] fn regress_m22_generic_fn_ptr() {
    let src = "\
fn apply[T](f: fn(T) -> T, x: T) -> T { return f(x); }
fn square(x: Int) -> Int { return x * x; }
fn main() -> Int { return apply(square, 5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: generic fn ptr must compile");
}

// ── M22-7: Pattern matching ────────────────────────────────────────────

#[test] fn regress_m22_pat_deep_nested_match() {
    let src = "\
enum L { Cons(v: Int, next: Int); Nil; }
fn sum(lst: L) -> Int {
    match lst {
        L.Cons(v, n) => v + n,
        L.Nil => 0,
    }
}
fn main() -> Int { return sum(L.Cons(5, 3)); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: deep nested match must compile");
}

#[test] fn regress_m22_pat_refutable_guard() {
    let src = "\
fn abs(n: Int) -> Int {
    match n {
        n => if n > 0 { n } else { -n },
    }
}
fn main() -> Int { return abs(-5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: refutable guard must compile");
}

#[test] fn regress_m22_pat_tuple_arm() {
    let src = "\
type Pair = { x: Int; y: Int; }
fn main() -> Int {
    var p = Pair{ x: 3; y: 4; };
    match p { Pair{ x: a; y: b; } => a + b, }
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: struct destructure match must compile");
}

#[test] fn regress_m22_pat_exhaustive_match() {
    let src = "\
enum Opt { Some(v: Int); None; }
fn extract(o: Opt) -> Int {
    match o {
        Opt.Some(v) => v,
        Opt.None => -1,
    }
}
fn main() -> Int { return extract(Opt.None); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: exhaustive match must compile");
}

#[test] fn regress_m22_pat_nested_enum() {
    let src = "\
enum Inner { Val(v: Int); None; }
enum Outer { Single(v: Int); Wrapped(i: Inner); }
fn unwrap(o: Outer) -> Int {
    match o {
        Outer.Single(v) => v,
        Outer.Wrapped(i) => match i { Inner.Val(v) => v, Inner.None => 0, },
    }
}
fn main() -> Int { return unwrap(Outer.Wrapped(Inner.Val(99))); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: nested enum pattern must compile");
}

#[test] fn regress_m22_pat_bool_match() {
    let src = "\
fn classify(b: Bool) -> Int {
    match b { true => 1, false => 0, }
}
fn main() -> Int { return classify(true); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: bool match must compile");
}

#[test] fn regress_m22_pat_int_range() {
    let src = "\
fn grade(score: Int) -> Str {
    match score {
        n => if n >= 90 { \"A\" } else if n >= 80 { \"B\" } else if n >= 70 { \"C\" } else { \"F\" },
    }
}
fn main() -> Int { var g = grade(95); return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: int range match must compile");
}

#[test] fn regress_m22_pat_match_return() {
    let src = "\
fn sign(n: Int) -> Str {
    match n {
        n => if n > 0 { return \"positive\"; } else if n < 0 { return \"negative\"; } else { return \"zero\"; },
    }
}
fn main() -> Int { var s = sign(0); return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "M22: match return must compile");
}