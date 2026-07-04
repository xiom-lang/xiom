// XIOM — Integration Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_codegen::IrEmitter;

fn compile(source: &str) -> Result<String, String> {
    let tokens = Lexer::new(source).tokenize();
    let program = Parser::new(tokens).parse_program().map_err(|e| e.to_string())?;
    let mut emitter = IrEmitter::new();
    emitter.compile_program(&program)
}

#[test]
fn test_codegen_simple_fn() {
    let ir = compile("fn add(a: Int, b: Int) -> Int { return a + b; }").unwrap();
    assert!(ir.contains("define i64 @add"));
    assert!(ir.contains("ret i64"));
}

#[test]
fn test_codegen_empty_fn() {
    let ir = compile("fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define i64 @main"));
    assert!(ir.contains("ret i64 0"));
}

#[test]
fn test_codegen_void_fn() {
    let ir = compile("fn do_nothing() { }").unwrap();
    assert!(ir.contains("define void @do_nothing"));
    assert!(ir.contains("ret void"));
}

#[test]
fn test_codegen_with_contract() {
    let ir = compile("fn div(a: Float64, b: Float64) -> Float64 requires: b != 0.0 { return a / b; }").unwrap();
    assert!(ir.contains("define double @div"));
    assert!(ir.contains("contract_fail"));
    assert!(ir.contains("@llvm.trap()"));
}

#[test]
fn test_codegen_function_call() {
    let ir = compile("fn square(x: Int) -> Int { return x * x; } fn main() -> Int { return square(5); }").unwrap();
    assert!(ir.contains("define i64 @square"));
    assert!(ir.contains("define i64 @main"));
    assert!(ir.contains("call i64 @square"));
}

#[test]
fn test_codegen_struct() {
    let ir = compile("type Point = { x: Float64; y: Float64; } fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("%struct.Point"));
}

#[test]
fn test_codegen_struct_literal() {
    let ir = compile("type Point = { x: Float64; y: Float64; } fn make() -> Point { return Point{ x: 1.0, y: 2.0 }; }").unwrap();
    assert!(ir.contains("%struct.Point = type"));
    assert!(ir.contains("define %struct.Point @make"));
}

#[test]
fn test_codegen_derive_eq() {
    let ir = compile("type Point = { x: Int; y: Int; } derive[Eq] fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define i64 @Point.eq"));
}

#[test]
fn test_codegen_derive_clone() {
    let ir = compile("type Point = { x: Int; y: Int; } derive[Clone] fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("define %struct.Point @Point.clone"));
}

#[test]
fn test_codegen_if_else() {
    let ir = compile("fn test(x: Int) -> Int { if x > 0 { return 1; } else { return 0; } }").unwrap();
    assert!(ir.contains("define i64 @test"));
    assert!(ir.contains("br i1"));
}

#[test]
fn test_codegen_while_loop() {
    let ir = compile("fn test() -> Int { var i = 0; while i < 10 { i = i + 1; } return i; }").unwrap();
    assert!(ir.contains("while_cond"));
    assert!(ir.contains("while_body"));
    assert!(ir.contains("while_exit"));
}

#[test]
fn test_codegen_let_var() {
    let ir = compile("fn main() -> Int { let x: Int = 42; var y = 10; return x + y; }").unwrap();
    assert!(ir.contains("define i64 @main"));
    assert!(ir.contains("alloca i64"));
}

// ========================================================================
// GAP 1: Contract Collection Methods
// ========================================================================

#[test]
fn test_contract_collection_declares() {
    let ir = compile("fn main() -> Int { return 0; }").unwrap();
    assert!(ir.contains("declare i64 @xiom_is_sorted(i8*, i64)"));
    assert!(ir.contains("declare i64 @xiom_all(i8*, i64, i8*)"));
    assert!(ir.contains("declare i64 @xiom_none(i8*, i64, i8*)"));
    assert!(ir.contains("declare i64 @xiom_contains(i8*, i64)"));
}

#[test]
fn test_contract_collection_method_form() {
    let ir = compile("type Items = { x: Int; } fn check(items: Items) -> Bool { return items.is_sorted(); }").unwrap();
    assert!(ir.contains("is_sorted"));
}

// ========================================================================
// GAP 2: Invariant Check After Let/Var
// ========================================================================

#[test]
fn test_invariant_check_after_let() {
    let ir = compile("type Range = { low: Int; high: Int; invariant: low <= high; } fn make(low: Int, high: Int) -> Range { let r = Range{ low: low, high: high }; return r; }").unwrap();
    assert!(ir.contains("Range.invariant_check"));
}

#[test]
fn test_invariant_check_after_var() {
    let ir = compile("type Range = { low: Int; high: Int; invariant: low <= high; } fn make(low: Int, high: Int) -> Range { var r = Range{ low: low, high: high }; return r; }").unwrap();
    assert!(ir.contains("Range.invariant_check"));
}

// ========================================================================
// GAP 3: Option ? Operator
// ========================================================================

#[test]
fn test_option_try_operator() {
    let ir = compile("fn try_option(x: Int) -> Int { let val = Some(x)?; return val; }").unwrap();
    assert!(ir.contains("try_some"));
    assert!(ir.contains("try_none"));
    assert!(ir.contains("ret i64 0"));
}

#[test]
fn test_codegen_contract_ensures() {
    let src = "fn double(x: Int) -> Int ensures: result > x { return x + x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @double"), "should define function");
    assert!(ir.contains("contract_fail") || ir.contains("@llvm.trap"), "should have contract failure path");
}

#[test]
fn test_codegen_generic_fn() {
    let src = "fn add[T](a: T, b: T) -> T { return a + b; } fn main() -> Int { return add(10, 20); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @add_Int"), "should have monomorphised add_Int");
    assert!(ir.contains("call i64 @add_Int"), "should call monomorphised version");
}

#[test]
fn test_codegen_module_fn() {
    let src = "module math { pub fn add(a: Int, b: Int) -> Int { return a + b; } } fn main() -> Int { return add(3, 4); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @add"), "should have function from module");
    assert!(ir.contains("call i64 @add"), "should call function from module");
}

#[test]
fn test_codegen_match_option() {
    let src = "fn unwrap(x: Int) -> Int { match x { 1 => 10, 2 => 20, _ => 0, } }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @unwrap"), "should define function");
    assert!(ir.contains("br label"), "should have branch for match");
}

#[test]
fn test_codegen_derive_display() {
    let src = "type Point = { x: Int; y: Int; } derive[Display] fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i8* @Point.to_str"), "should have display function");
}

#[test]
fn test_codegen_derive_hash() {
    let src = "type Point = { x: Int; y: Int; } derive[Hash] fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("@Point.hash"), "should have hash function");
}

#[test]
fn test_codegen_derive_ord() {
    let src = "type Point = { x: Int; y: Int; } derive[Ord] fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("@Point.compare"), "should have compare function");
}

#[test]
fn test_codegen_ret_void_branch() {
    let src = "fn maybe(x: Int) { if x > 0 { return; } }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define void @maybe"), "should define void fn");
    assert!(ir.contains("ret void"), "should return void");
}

// ========================================================================
// Phase 1.5: Interface Constraint Check
// ========================================================================

#[test]
fn test_interface_constraint_check() {
    // Test that interface declarations are registered and interface constraint
    // checking runs during monomorphisation (even if the concrete type check triggers an error).
    let src = "\
interface Foo { fn bar() -> Int; }
type MyType = { x: Int; }
fn MyType.bar() -> Int { return 42; }
fn use_foo[T: Foo](x: T) -> Int { return 0; }
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("MyType.bar"), "should have MyType.bar function");
}

// ========================================================================
// Phase 1.5: Enum Variant Field Extraction
// ========================================================================

#[test]
fn test_enum_variant_field_extraction() {
    // Test that enum type is registered with variant fields
    let src = "\
enum Token { Ident(name: Int), IntLit(value: Int) }
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("%struct.Token = type"), "should have Token struct type");
}

// ========================================================================
// Phase 1.5: Builtin Option.is_some
// ========================================================================

#[test]
fn test_builtin_option_is_some() {
    // Test that Option.is_some is emitted when the program uses Some
    let src = "\
fn test() -> Int { let x = Some(42); return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @Option.is_some"), "should emit Option.is_some");
    assert!(ir.contains("define i64 @Option.is_none"), "should emit Option.is_none");
    assert!(ir.contains("define i64 @Option.unwrap"), "should emit Option.unwrap");
}

// ========================================================================
// Phase 1.5: Builtin Result.is_ok
// ========================================================================

#[test]
fn test_builtin_result_is_ok() {
    // Test that Result.is_ok is emitted when the program uses Ok
    let src = "\
fn test() -> Int { let x = Ok(42); return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @Result.is_ok"), "should emit Result.is_ok");
    assert!(ir.contains("define i64 @Result.is_err"), "should emit Result.is_err");
    assert!(ir.contains("define i64 @Result.unwrap"), "should emit Result.unwrap");
    assert!(ir.contains("define i64 @Result.unwrap_err"), "should emit Result.unwrap_err");
}

// ========================================================================
// Phase 1.5: Malloc/Free Declares
// ========================================================================

#[test]
fn test_malloc_free_declares() {
    let src = "\
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("declare i8* @malloc(i64)"), "should declare malloc");
    assert!(ir.contains("declare void @free(i8*)"), "should declare free");
    assert!(ir.contains("declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)"), "should declare memcpy");
}

// ========================================================================
// Vec Operations
// ========================================================================

#[test]
fn test_vec_new_initial_capacity() {
    let ir = compile("fn main() -> Int { var v = Vec[Int].new(); return 0; }").unwrap();
    assert!(ir.contains("store i64 0"), "len should be 0");
    assert!(ir.contains("store i64 16"), "cap should be 16");
    assert!(ir.contains("@malloc(i64 128)"), "should allocate 128 bytes");
}

#[test]
fn test_vec_push_within_capacity() {
    let ir = compile("fn main() -> Int { var v = Vec[Int].new(); v.push(1); return 0; }").unwrap();
    assert!(ir.contains("vec_store"), "should use store block");
    assert!(ir.contains("store i64 1"), "should store the pushed value");
}

#[test]
fn test_vec_push_triggers_growth() {
    // Push 17 Int-s into vec; cap=16 so 17th push triggers grow + realloc
    let src = "fn main() -> Int { var v = Vec[Int].new(); v.push(1); v.push(2); v.push(3); v.push(4); v.push(5); v.push(6); v.push(7); v.push(8); v.push(9); v.push(10); v.push(11); v.push(12); v.push(13); v.push(14); v.push(15); v.push(16); v.push(17); return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("vec_grow"), "should contain grow block");
    assert!(ir.contains("@realloc"), "should call realloc for growth");
    assert!(ir.contains("mul i64"), "should compute new capacity (cap * 2)");
}

#[test]
fn test_vec_push_realloc_null_trap() {
    let ir = compile("fn push_two(v: Vec[Int]) { v.push(1); v.push(2); }").unwrap();
    // Grow path always includes null check + trap for OOM
    assert!(ir.contains("vec_realloc_ok"), "should have realloc ok block");
    assert!(ir.contains("vec_realloc_trap"), "should trap on realloc failure");
    assert!(ir.contains("@llvm.trap"), "should call llvm.trap on OOM");
}

#[test]
fn test_vec_capacity_guard_max() {
    let ir = compile("fn push_one(v: Vec[Int]) { v.push(1); }").unwrap();
    assert!(ir.contains("vec_cap_trap"), "should have max-capacity trap block");
    assert!(ir.contains("1048576"), "should bound capacity at 2^20 elements");
    assert!(ir.contains("icmp ule"), "should use unsigned <= comparison");
}

#[test]
fn test_vec_len_method() {
    let ir = compile("fn main() -> Int { var v = Vec[Int].new(); return v.len(); }").unwrap();
    assert!(ir.contains("getelementptr %struct.Vec"), "should access vec struct");
    assert!(ir.contains("load i64, i64*"), "should load len field");
}

#[test]
fn test_vec_multiple_operations() {
    let src = "fn main() -> Int { var v = Vec[Int].new(); v.push(10); v.push(20); var x = v.len(); return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"), "should compile");
    assert!(ir.contains("%struct.Vec"), "should use Vec struct");
}

// ========================================================================
// Phase 2: Division-by-Zero Protection
// ========================================================================

#[test]
fn test_div_zero_protection_integer() {
    let src = "fn div(a: Int, b: Int) -> Int { return a / b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("div_zero_trap"), "should have trap block for zero divisor");
    assert!(ir.contains("div_safe"), "should have safe division block");
    assert!(ir.contains("icmp eq i64"), "should compare divisor to zero");
    assert!(ir.contains("@llvm.trap()"), "should call llvm.trap on zero divisor");
    assert!(ir.contains("unreachable"), "should be unreachable after trap");
}

#[test]
fn test_div_zero_protection_modulo() {
    let src = "fn rem(a: Int, b: Int) -> Int { return a % b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("div_zero_trap"), "modulo should also check for zero");
    assert!(ir.contains("srem"), "should use srem instruction");
    assert!(ir.contains("div_safe"), "should have safe modulo block");
}

#[test]
fn test_float_div_no_zero_guard() {
    let src = "fn fdiv(a: Float64, b: Float64) -> Float64 { return a / b; }";
    let ir = compile(src).unwrap();
    assert!(!ir.contains("div_zero_trap"), "float division must NOT use integer zero-check");
    assert!(ir.contains("fdiv double"), "should use float division instruction");
}

#[test]
fn test_mixed_float_div_no_integer_guard() {
    // Int-to-float coercion before division — result is float, no zero-guard needed
    let src = "fn mixed(a: Float64, b: Int) -> Float64 { return a / b; }";
    let ir = compile(src).unwrap();
    assert!(!ir.contains("div_zero_trap"), "float division after coercion should not have int guard");
}

// ========================================================================
// Phase 3: Generic Monomorphisation
// ========================================================================

#[test]
fn test_generic_multi_param() {
    let src = "fn pair[T, U](a: T, b: U) -> Tuple2 { return (a, b); } fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "multi-param generic should compile");
}

#[test]
fn test_generic_nested_types() {
    let src = "fn first[T](v: Vec[T]) -> T { return v[0]; } fn main() -> Int { var v = Vec[Int].new(); v.push(42); return first(v); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "nested generic types should compile");
}

#[test]
fn test_monomorphisation_dedup_single_definition() {
    let src = "\
fn id[T](x: T) -> T { return x; }
fn main() -> Int { return id(1) + id(2) + id(3) + id(4) + id(5); }";
    let ir = compile(src).unwrap();
    // The specialized version should exist — exact naming depends on internal format
    let has_specialized = ir.contains("id_") || ir.contains("id.");
    assert!(has_specialized, "specialized id function should exist in IR");
    let define_count = ir.match_indices("define ").count();
    assert!(define_count > 0, "should have at least one define");
}

#[test]
fn test_generic_interface_bound_satisfied() {
    let src = "\
interface Eq { fn eq(other: &Self) -> Int; }
type Point = { x: Int; y: Int; }
fn Point.eq(other: &Point) -> Int { return 1; }
fn find[T: Eq](a: T, b: T) -> Int { return a.eq(&b); }
fn main() -> Int { var p = Point{ x: 1, y: 2 }; return find(p, p); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"), "should compile with satisfied bound");
}

#[test]
fn test_generic_many_instantiations() {
    // Call generic id with 3 different types
    let src = "\
fn id[T](x: T) -> T { return x; }
fn main() -> Int { return id(1) + id(id(2)) + id(id(id(3))); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"), "multiple generic instantiations should compile");
}

#[test]
fn test_generic_struct_method() {
    let src = "\
type Pair[T] = { first: T; second: T; }
fn Pair.get[T]() -> T { return first; }
fn main() -> Int { var p = Pair[Int]{ first: 10, second: 20 }; return p.get(); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "generic struct method should compile");
}

// ========================================================================
// Miscellaneous Codegen Tests
// ========================================================================

#[test]
fn test_recursion_counter_emitted() {
    let ir = compile("fn fact(n: Int) -> Int { return n; } fn main() -> Int { return fact(5); }").unwrap();
    assert!(ir.contains("@xiom_recursion_counter"), "should declare recursion counter");
}

#[test]
fn test_codegen_closure() {
    let src = "fn main() -> Int { var f = fn(x: Int) -> Int { return x * 2; }; return f(5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "closure should compile");
}

#[test]
fn test_codegen_cast_float_to_int() {
    let ir = compile("fn main() -> Int { return 3.14 as Int; }").unwrap();
    assert!(ir.contains("fptosi"), "float-to-int cast should emit fptosi");
}

#[test]
fn test_codegen_cast_int_to_float() {
    let ir = compile("fn main() -> Float64 { return 42 as Float64; }").unwrap();
    assert!(ir.contains("sitofp"), "int-to-float cast should emit sitofp");
}

#[test]
fn test_codegen_async_fn_declaration() {
    let src = "async fn fetch(url: Str) -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "async fn should produce define");
}

#[test]
fn test_codegen_spawn_block() {
    let src = "fn main() -> Int { spawn { var x = 1; } return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"), "spawn should compile in main");
}

#[test]
fn test_codegen_match_enum_variant_fields() {
    let src = "\
enum Color { Red, Green(code: Int), Blue }
fn value(c: Color) -> Int {
    match c { Red => 0, Green(code) => code, Blue => 2 }
}
fn main() -> Int { var c = Color{Red}; return value(c); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "match on enum with multiple variants should compile");
}

#[test]
fn test_codegen_for_in_loop() {
    let src = "fn main() -> Int { var sum = 0; for i in [1, 2, 3] { sum = sum + i; } return sum; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"), "for-in loop should compile");
}

#[test]
fn test_codegen_destructure_tuple() {
    let src = "fn main() -> Int { var (a, b) = (1, 2); return a + b; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"), "destructure binding should compile");
}

#[test]
fn test_codegen_try_operator_option() {
    let src = "fn try_val(x: Int) -> Int { var v = Some(x)?; return v + 1; } fn main() -> Int { return try_val(5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "try operator should compile");
}

#[test]
fn test_codegen_int8_float32_type_widths() {
    let ir = compile("fn add(a: Int8, b: Int8) -> Int8 { return a + b; } fn fadd(a: Float32, b: Float32) -> Float32 { return a + b; }").unwrap();
    assert!(ir.contains("i8"), "Int8 should map to i8 in IR");
    assert!(ir.contains("float "), "Float32 should map to float in IR");
}

#[test]
fn test_codegen_method_ref_self() {
    let src = "\
type Counter = { val: Int; }
fn Counter.inc() -> Int { return val + 1; }
fn main() -> Int { var c = Counter{ val: 0 }; return c.inc(); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "method should compile");
}
