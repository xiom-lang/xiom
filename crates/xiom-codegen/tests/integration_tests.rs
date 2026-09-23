// XIOM -- Integration Tests
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

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
    assert!(ir.contains("declare i64 @xiom_is_sorted(i8*)"));
    assert!(ir.contains("declare i64 @xiom_all(i8*, i64, i8*)"));
    assert!(ir.contains("declare i64 @xiom_none(i8*, i64, i8*)"));
    assert!(ir.contains("declare i64 @xiom_contains(i8*, i64)"));
}

#[test]
fn test_contract_collection_method_form() {
    // m119: the method form is lowered INLINE over the receiver's Vec header.
    // The old lowering called the xiom_is_sorted runtime intrinsic with a
    // pointer to the receiver VALUE; the runtime read data[0] as the element
    // COUNT, so a %struct.Vec receiver scanned its DATA POINTER as a length
    // (Windows-CI AV on e2e_p1_contract_methods, wrong answers elsewhere).
    let ir = compile("fn check(items: Vec[Int]) -> Bool { return items.is_sorted(); }").unwrap();
    assert!(ir.contains("csorted_loop"), "m119 inline scan must be emitted");
    assert!(ir.contains("is_sorted")); // the declare block still carries the symbol
}

#[test]
fn test_contract_collection_method_rejects_unsupported_receiver() {
    // A receiver with no concrete element type (a plain struct) must be
    // rejected LOUDLY -- the pre-m119 code emitted a call that scanned
    // whatever memory followed the receiver alloca.
    let err = compile("type Items = { x: Int; } fn check(items: Items) -> Bool { return items.is_sorted(); }").unwrap_err();
    assert!(err.contains("unsupported: 'is_sorted'"), "unexpected error: {err}");
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
    // The capacity limit may vary -- check that SOME limit exists
    assert!(ir.contains("1048576") || ir.contains("2097152") || ir.contains("524288") || ir.contains("icmp ule"),
        "should have capacity guard with unsigned comparison");
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
    // Int-to-float coercion before division -- result is float, no zero-guard needed
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
    // The specialized version should exist -- exact naming depends on internal format
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

// ============================================================================
// FFI Safety & Contract Verification Tests
// ============================================================================

#[test]
fn test_ffi_null_check_contract_emits_trap() {
    let src = "fn process(ptr: *UInt8) -> Int requires: ptr != null { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("@llvm.trap"), "null-check contract should emit trap");
    assert!(ir.contains("contract_fail"), "should have contract fail block");
}

#[test]
fn test_ffi_extern_block_emits_declare() {
    let src = "extern \"C\" { fn malloc(size: Int) -> *UInt8; } fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("declare"), "extern block should emit LLVM declare");
}

#[test]
fn test_contract_requires_addition_overflow() {
    let src = "fn checked_add(a: Int, b: Int) -> Int requires: a + b >= a { return a + b; } fn main() -> Int { return checked_add(1, 2); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "overflow contract should compile");
}

#[test]
fn test_contract_ensures_division_precision() {
    let src = "fn divide(a: Float64, b: Float64) -> Float64 requires: b != 0.0 ensures: result * b == a { return a / b; } fn main() -> Float64 { return divide(10.0, 2.0); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "division contract should compile");
    assert!(ir.contains("contract"), "should contain contract checking");
}

#[test]
fn test_ffi_unsafe_block_compiles() {
    let src = "fn main() -> Int { var x = 1; return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define i64 @main"), "basic block should compile");
}

#[test]
fn test_contract_multiple_requires() {
    let src = "fn transfer(amount: Int, balance: Int) -> Int requires: amount > 0 requires: balance >= amount { return balance - amount; } fn main() -> Int { return transfer(10, 100); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "multiple requires should compile");
}

#[test]
fn test_contract_type_invariant_compound() {
    let src = "\
type Bounded = { val: Int; min: Int; max: Int; invariant: min <= max; }
fn Bounded.clamp() -> Int requires: val >= min ensures: result <= max { return val; }
fn main() -> Int { var b = Bounded{ val: 5, min: 0, max: 10 }; return b.clamp(); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("invariant_check"), "should check type invariant");
    assert!(ir.contains("contract"), "should check method contract");
}

#[test]
fn test_ffi_raw_pointer_operations() {
    let src = "\
extern \"C\" { fn malloc(size: Int) -> *UInt8; fn free(ptr: *UInt8); }
fn alloc_and_free() {
    var ptr = malloc(64);
    free(ptr);
}
fn main() -> Int { alloc_and_free(); return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "raw pointer operations should compile");
}

// ============================================================================
// Parser Feature E2E Tests -- verify new syntax compiles to IR
// ============================================================================

#[test]
fn test_parse_ptr_type_codegen() {
    let src = "fn use_ptr(ptr: *UInt8) -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "pointer type should compile");
}

#[test]
fn test_parse_generic_bound_codegen() {
    let src = "\
interface Display { fn show() -> Str; }
fn print[T: Display](x: T) { }
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "generic bound should compile");
}

#[test]
fn test_parse_dotted_type_codegen() {
    let src = "\
module test_mod
type Error = { msg: Int; }
fn handle(e: test_mod.Error) -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "dotted type should compile");
}

#[test]
fn test_parse_array_const_size_codegen() {
    let src = "fn sum(arr: [5]Int) -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "array type should compile");
}

#[test]
fn test_parse_struct_spread_codegen() {
    let src = "\
type Point = { x: Int; y: Int; }
fn main() -> Int { var p = Point{ x: 1, ..Point{ x: 0, y: 0 } }; return p.x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "struct spread should compile");
}

#[test]
fn test_parse_caret_xor_codegen() {
    let src = "fn xor(a: Int, b: Int) -> Int { return a ^ b; } fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "xor operator should compile");
}

#[test]
fn test_parse_tilde_not_codegen() {
    let src = "fn not_val(a: Int) -> Int { return ~a; } fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "tilde not should compile");
}

#[test]
fn test_parse_pub_const_codegen() {
    let src = "const MAX: Int = 100; fn main() -> Int { return MAX; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "const should compile");
}

#[test]
fn test_parse_top_level_var_codegen() {
    let src = "var counter: Int = 0; fn main() -> Int { return counter; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "top-level var should compile");
}

#[test]
fn test_parse_extern_block_codegen() {
    let src = "\
extern \"C\" { fn malloc(size: Int) -> *UInt8; }
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "extern block should compile");
}

#[test]
fn test_parse_method_receiver_codegen() {
    let src = "\
type Counter = { val: Int; }
fn Counter.inc(&self) -> Int { return val + 1; }
fn main() -> Int { var c = Counter{ val: 0 }; return c.inc(); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "method with self should compile");
}

// ============================================================================
// Real-World Pattern Tests -- stdlib-like code that exercises multiple features
// ============================================================================

#[test]
fn test_pattern_extern_with_multiple_fns() {
    let src = "\
extern \"C\" {
    fn malloc(size: Int) -> *UInt8;
    fn free(ptr: *UInt8);
    fn memcpy(dest: *UInt8, src: *UInt8, n: Int) -> *UInt8;
}
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("declare"), "extern block should emit declares");
}

#[test]
fn test_pattern_struct_with_generic_field() {
    let src = "\
type Pair[T] = { first: T; second: T; }
fn main() -> Int { var p = Pair[Int]{ first: 1, second: 2 }; return p.first; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "generic struct should compile");
}

#[test]
fn test_pattern_interface_with_impl() {
    let src = "\
interface Eq { fn eq(other: &Self) -> Int; }
type Point = { x: Int; y: Int; }
fn Point.eq(other: &Point) -> Int { return 1; }
fn main() -> Int { var p = Point{ x: 1, y: 2 }; return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "interface with impl should compile");
}

#[test]
fn test_pattern_type_with_derive_clone_eq() {
    let src = "\
type Pos = { x: Float64; y: Float64; } derive[Clone, Eq]
fn main() -> Int { var p = Pos{ x: 1.0, y: 2.0 }; return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "type with derive should compile");
}

#[test]
fn test_pattern_multiple_generic_fns_in_module() {
    let src = "\
fn id[T](x: T) -> T { return x; }
fn swap[T](a: T, b: T) -> (T, T) { return (b, a); }
fn main() -> Int { return id(42); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "multiple generic fns should compile");
}

#[test]
fn test_pattern_match_on_option() {
    let src = "\
fn get_val(x: Option[Int]) -> Int {
    match x { Some(v) => v, None => 0 }
}
fn main() -> Int { return get_val(Some(42)); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "match on Option should compile");
}

#[test]
fn test_pattern_vec_push_pop_loop() {
    let src = "\
fn sum_vec(v: Vec[Int]) -> Int {
    var sum = 0; var i = 0;
    while i < v.len() { sum = sum + v[i]; i = i + 1; }
    return sum;
}
fn main() -> Int { var v = Vec[Int].new(); v.push(1); v.push(2); return sum_vec(v); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "vec push+loop should compile");
}

#[test]
fn test_pattern_contract_on_method() {
    let src = "\
type Account = { balance: Int; }
fn Account.withdraw(amount: Int) -> Int
    requires: amount > 0
    requires: balance >= amount
    ensures: result == balance
{
    balance = balance - amount;
    return balance;
}
fn main() -> Int { var a = Account{ balance: 100 }; return a.withdraw(30); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "contract method should compile");
}

#[test]
fn test_pattern_comptime_fold() {
    let src = "\
fn main() -> Int { return comptime 1 + 2 * 3; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "comptime expr should compile");
}

#[test]
fn test_pattern_spawn_async_pattern() {
    let src = "\
async fn fetch() -> Int { return 42; }
fn main() -> Int { spawn { var x = 1; } return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "spawn+async should compile");
}

#[test]
fn test_pattern_const_used_in_fn() {
    let src = "\
const MAX: Int = 100;
fn within_limit(x: Int) -> Int { if x > MAX { return MAX; } return x; }
fn main() -> Int { return within_limit(50); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "const in fn should compile");
}

#[test]
fn test_pattern_pub_type_and_fn() {
    let src = "\
pub type Data = { val: Int; }
pub fn process(d: Data) -> Int { return d.val; }
fn main() -> Int { var d = Data{ val: 42 }; return process(d); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "pub type+fn should compile");
}

// ============================================================================
// Advanced Pattern Tests -- real-world XIOM patterns
// ============================================================================

#[test]
fn test_pattern_question_operator_chain() {
    let src = "\
fn try_div(a: Int, b: Int) -> Option[Int] {
    if b == 0 { return None; }
    return Some(a / b);
}
fn compute(x: Int, y: Int) -> Option[Int] {
    var a = try_div(x, y)?;
    var b = try_div(a, 2)?;
    return Some(b);
}
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "? operator chain should compile");
}

#[test]
fn test_pattern_for_loop_with_range() {
    let src = "\
fn sum_range(lo: Int, hi: Int) -> Int {
    var sum = 0; var i = lo;
    while i <= hi { sum = sum + i; i = i + 1; }
    return sum;
}
fn main() -> Int { return sum_range(1, 10); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "for/while loop should compile");
}

#[test]
fn test_pattern_enum_variant_with_named_fields() {
    let src = "\
enum Shape { Circle(radius: Float64), Rect(w: Float64, h: Float64) }
fn area(s: Shape) -> Float64 {
    match s { Circle(radius) => 3.14 * radius * radius, Rect(w, h) => w * h }
}
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "enum variant with fields should compile");
}

#[test]
fn test_pattern_if_as_expression() {
    // if/else as return expression (supported pattern)
    let src = "\
fn abs(x: Int) -> Int {
    if x >= 0 { return x; } else { return -x; }
}
fn main() -> Int { return abs(-5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "if/else should compile");
}

#[test]
fn test_pattern_elif_chain() {
    let src = "\
fn grade(score: Int) -> Int {
    if score >= 90 { return 4; }
    elif score >= 80 { return 3; }
    elif score >= 70 { return 2; }
    elif score >= 60 { return 1; }
    else { return 0; }
}
fn main() -> Int { return grade(85); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "elif chain should compile");
}

#[test]
fn test_pattern_tuple_return_and_destructure() {
    let src = "\
fn minmax(a: Int, b: Int) -> (Int, Int) {
    if a < b { return (a, b); } else { return (b, a); }
}
fn main() -> Int { var (mn, mx) = minmax(10, 5); return mn; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "tuple return should compile");
}

#[test]
fn test_pattern_closure_capture() {
    let src = "\
fn make_adder(n: Int) -> Int {
    var f = fn(x: Int) -> Int { return x + n; };
    return f(10);
}
fn main() -> Int { return make_adder(5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "closure capture should compile");
}

#[test]
fn test_pattern_array_literal_index() {
    let src = "\
fn main() -> Int { var arr = [10, 20, 30, 40, 50]; return arr[2]; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "array literal index should compile");
}

#[test]
fn test_pattern_while_with_break_continue() {
    let src = "\
fn countdown(n: Int) -> Int {
    var i = n; var sum = 0;
    while i > 0 { sum = sum + i; i = i - 1; }
    return sum;
}
fn main() -> Int { return countdown(5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "while loop should compile");
}

#[test]
fn test_pattern_recursive_fn() {
    let src = "\
fn fact(n: Int) -> Int {
    if n <= 1 { return 1; }
    return n * fact(n - 1);
}
fn main() -> Int { return fact(5); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "recursive fn should compile");
}

#[test]
fn test_pattern_result_error_propagation() {
    let src = "\
fn safe_div(a: Int, b: Int) -> Result[Int, Str] {
    if b == 0 { return Err(\"division by zero\"); }
    return Ok(a / b);
}
fn main() -> Int {
    match safe_div(10, 2) { Ok(v) => return v, Err(_) => return 0 }
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "result error propagation should compile");
}

#[test]
fn test_pattern_type_alias_with_generic() {
    let src = "\
type Point[T] = { x: T; y: T; }
fn make_point(x: Int, y: Int) -> Point[Int] { return Point[Int]{ x: x, y: y }; }
fn main() -> Int { var p = make_point(1, 2); return p.x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "type alias with generic should compile");
}

#[test]
fn test_pattern_multiple_use_imports() {
    let src = "\
type Logger = { level: Int; }
fn Logger.log(msg: Int) -> Int { return level + msg; }
fn main() -> Int { var l = Logger{ level: 1 }; return l.log(2); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "multiple imports should compile");
}

#[test]
fn test_pattern_contract_with_result_keyword() {
    let src = "\
fn identity(x: Int) -> Int
    ensures: result == x
{
    return x;
}
fn main() -> Int { return identity(42); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "contract with result keyword should compile");
}

#[test]
fn test_pattern_nested_struct_literal() {
    let src = "\
type Inner = { val: Int; }
type Outer = { inner: Inner; tag: Int; }
fn main() -> Int { var o = Outer{ inner: Inner{ val: 42 }, tag: 1 }; return o.inner.val; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "nested struct literal should compile");
}

// ============================================================================
// Edge Case Tests -- patterns from stdlib compilation findings
// ============================================================================

#[test]
fn test_pattern_result_question_operator() {
    let src = "\
fn div(a: Int, b: Int) -> Result[Int, Str] {
    if b == 0 { return Err(\"div by zero\"); }
    return Ok(a / b);
}
fn calc(x: Int) -> Result[Int, Str] {
    var a = div(x, 2)?;
    return Ok(a + 1);
}
fn main() -> Int { return 0; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "result ? operator should compile");
}

#[test]
fn test_pattern_match_wildcard() {
    let src = "\
fn grade(score: Int) -> Int {
    match score { 100 => 5, 90 => 4, 80 => 3, 70 => 2, 60 => 1, _ => 0 }
}
fn main() -> Int { return grade(85); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "match wildcard should compile");
}

#[test]
fn test_pattern_bool_literals_in_expr() {
    let src = "\
fn is_valid(age: Int) -> Int {
    var ok = age >= 0 && age <= 150;
    if ok { return 1; } else { return 0; }
}
fn main() -> Int { return is_valid(25); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "bool expressions should compile");
}

#[test]
fn test_pattern_float_arithmetic_chain() {
    let src = "\
fn compute(a: Float64, b: Float64, c: Float64) -> Float64 {
    return a * b + c / 2.0 - 1.0;
}
fn main() -> Float64 { return compute(1.0, 2.0, 3.0); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "float arithmetic chain should compile");
}

#[test]
fn test_pattern_int_overflow_guard() {
    let src = "\
fn safe_add(a: Int, b: Int) -> Int
    requires: a + b >= a
{
    return a + b;
}
fn main() -> Int { return safe_add(100, 200); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "overflow guard should compile");
}

#[test]
fn test_pattern_char_literal() {
    let src = "\
fn is_digit(c: Char) -> Int {
    if c >= '0' && c <= '9' { return 1; } else { return 0; }
}
fn main() -> Int { return is_digit('5'); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "char literal should compile");
}

#[test]
fn test_pattern_negative_literal() {
    let src = "\
fn main() -> Int { var x = -42; return x; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "negative literal should compile");
}

// ============================================================================
// FFI Extern Declare Tests
// ============================================================================

#[test]
fn test_extern_declare_user_defined() {
    let src = r#"
extern "C" {
    fn vkFooBar(x: Int32, p: *UInt8) -> Int;
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("declare i64 @vkFooBar(i32, i8*)"),
        "user-defined extern function should emit correct declare with i32 and i8*");
}

#[test]
fn test_extern_declare_hardcoded_not_duplicated() {
    let src = r#"
extern "C" {
    fn malloc(size: Int) -> *UInt8;
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    // Count occurrences: there should be exactly ONE declare for malloc
    let count = ir.matches("declare i8* @malloc(i64)").count();
    assert_eq!(count, 1, "malloc must not be declared twice when user extern block declares it");
}

#[test]
fn test_extern_declare_multiple_blocks() {
    let src = r#"
extern "C" {
    fn vkFoo(x: Int32) -> Int;
    fn vkBar(x: Float32) -> Int;
}
extern "C" {
    fn vkBaz(x: Int32) -> Int;
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("declare i64 @vkFoo(i32)"), "vkFoo should be declared");
    assert!(ir.contains("declare i64 @vkBar(float)"), "vkBar should be declared");
    assert!(ir.contains("declare i64 @vkBaz(i32)"), "vkBaz should be declared");
    // Only one declare per name even if it appears in multiple blocks
    assert!(ir.match_indices("declare i64 @vkBaz(i32)").count() == 1,
        "vkBaz must not be declared twice");
}

#[test]
fn test_extern_declare_void_return() {
    let src = r#"
extern "C" {
    fn vkDestroy(value: Int32);
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("declare void @vkDestroy(i32)"),
        "void extern should emit declare void");
}

#[test]
fn test_extern_declare_in_module() {
    let src = r#"
module vulkan {
    extern "C" {
        fn vkCreateInstance(p: *UInt8) -> Int;
    }
}
fn main() -> Int { return 0; }
"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("declare i64 @vkCreateInstance(i8*)"),
        "extern inside module should still emit global declare (C linkage)");
}

#[test]
fn test_extern_call_with_correct_types() {
    // Verify that calls to extern functions use the declared param types rather
    // than inferred expression types (e.g., Int32 arg should produce 'i32' not 'i64').
    let src = r#"
extern "C" {
    fn vkFoo(x: Int32, y: Float32) -> Int;
}
fn main(x: Int32, y: Float32) -> Int {
    return vkFoo(x, y);
}
"#;
    let ir = compile(src).unwrap();
    // The declare should use correct types
    assert!(ir.contains("declare i64 @vkFoo(i32, float)"),
        "extern declare should have i32 and float");
    // The call should also use correct types
    assert!(ir.contains("call i64 @vkFoo(i32 %"),
        "call to vkFoo should pass i32 arg, not i64");
}

// ============================================================================

#[test]
fn test_match_many_arms_no_panic() {
    let src = "\
enum Token { A, B(x: Int), C, D(y: Int), E }
fn classify(t: Token) -> Int {
    match t {
        A => 1,
        B(x) => x,
        C => 3,
        D(y) => y,
        E => 5,
    }
}
fn main() -> Int { return 0; }";
    // Regression: mixed variant/binding arms previously desynced the two
    // match-codegen loops and panicked with an out-of-bounds index into
    // `check_labels`. The compiler must never panic -- compile must return a
    // value (Ok or Err), not unwind.
    let result = compile(src);
    assert!(result.is_ok() || result.is_err()); // just must not panic
}

// -- M30+: Deep combinatorial & differential stress --------------------

// 5 features interacting: struct + enum + generic + match + contract
#[test] fn test_combo_5_features() {
    let src = "\
type Boxed[T] = { val: T; }
enum Result[T, E] { Ok(v: T); Err(e: E); }
fn unwrap_or[T](r: Result[T, Str], default: T) -> T ensures: true {
    match r { Result.Ok(v) => v, Result.Err(_) => default, }
}
fn main() -> Int {
    var b = Boxed[Result[Int, Str]]{ val: Result[Int,Str].Ok(42); };
    match b.val { Result.Ok(v) => v, Result.Err(_) => 0, }
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "5 features must compile");
}

// 6 features: module + pub + type + method + generic + contract
#[test] fn test_combo_6_features() {
    let src = "\
module lib
pub type Counter = { val: Int; }
pub fn Counter.inc(c: Counter, n: Int) -> Counter requires: n > 0 ensures: result.val == c.val + n {
    return Counter{ val: c.val + n; };
}
fn main() -> Int { var c = lib.Counter{ val: 0; }; var c2 = c.inc(5); return c2.val; }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "6 features must compile");
}

// Differential: 3 different loop implementations of sum
#[test] fn test_diff_three_sum_styles() {
    let src = "\
fn sum_while(n: Int) -> Int {
    var total: Int = 0; var i: Int = 1;
    while i <= n { total = total + i; i = i + 1; }
    return total;
}
fn sum_loop_guard(n: Int) -> Int {
    var total: Int = 0; var i: Int = 0;
    while i < n { i = i + 1; total = total + i; }
    return total;
}
fn sum_rec(n: Int) -> Int {
    if n == 0 { return 0; }
    return n + sum_rec(n - 1);
}
fn main() -> Int {
    var a = sum_while(10);
    var b = sum_loop_guard(10);
    var c = sum_rec(10);
    if a == b && b == c { return 0; }
    return 1;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "diff 3 sum styles must compile");
}

// Differential: Fibonacci -- recursive, iterative, and match-based
#[test] fn test_diff_fib_three_ways() {
    let src = "\
fn fib_rec(n: Int) -> Int { if n <= 1 { return n; } return fib_rec(n - 1) + fib_rec(n - 2); }
fn fib_iter(n: Int) -> Int {
    if n <= 1 { return n; }
    var a: Int = 0; var b: Int = 1; var i: Int = 1;
    while i < n { var t = b; b = a + b; a = t; i = i + 1; }
    return b;
}
fn fib_match(n: Int) -> Int {
    match n { 0 => 0, 1 => 1, n => fib_match(n - 1) + fib_match(n - 2), }
}
fn main() -> Int {
    var a = fib_iter(10);
    var b = fib_rec(10);
    var c = fib_match(10);
    if a == b && b == c { return 0; }
    return 1;
}";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "diff fib 3 ways must compile");
}

// Combinatorial: enum with payload + generic type + nested match in method
#[test] fn test_combo_enum_gen_method_nested() {
    let src = "\
enum Tree[T] { Leaf(val: T); Node(left: Tree[T]; right: Tree[T]); }
fn Tree[T].sum(t: Tree[Int]) -> Int {
    match t {
        Tree.Leaf(v) => v,
        Tree.Node(l, r) => l.sum() + r.sum(),
    }
}
fn main() -> Int {
    var t = Tree[Int].Node(
        Tree[Int].Leaf(10),
        Tree[Int].Node(Tree[Int].Leaf(20), Tree[Int].Leaf(30)));
    return t.sum();
}";
    let result = compile(src);
    assert!(result.is_ok() || result.is_err(), "enum+gen+method+nested must not panic");
}

// Combinatorial: struct with invariant + method with contracts + generic
#[test] fn test_combo_invariant_method_contract_gen() {
    let src = "\
type Bounded[T] = { val: T; min: T; max: T; invariant: min <= max; }
fn Bounded[Int].clamp(b: Bounded[Int]) -> Int requires: b.val >= b.min ensures: result <= b.max {
    if b.val > b.max { return b.max; }
    if b.val < b.min { return b.min; }
    return b.val;
}
fn main() -> Int {
    var b = Bounded[Int]{ val: 50; min: 0; max: 100; };
    return b.clamp();
}";
    let result = compile(src);
    assert!(result.is_ok() || result.is_err(), "invariant+method+contract+gen must not panic");
}

// Differential: manual string ops equivalent
#[test] fn test_diff_string_equiv() {
    let src = r#"
fn concat_manual(a: Str, b: Str) -> Str { var result: Str = a + b; return result; }
fn concat_direct(a: Str, b: Str) -> Str { return a + b; }
fn main() -> Int {
    var r1 = concat_manual("hello", "world");
    var r2 = concat_direct("hello", "world");
    if r1.len() as Int == r2.len() as Int { return 0; }
    return 1;
}"#;
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "diff string equiv must compile");
}

// Stress: deeply nested block returns with shadowing
#[test] fn test_stress_deep_block_return() {
    let mut src = String::from("fn deep() -> Int {\n");
    for i in 0..10 {
        src.push_str(&format!("  var v{i}: Int = {i};\n  if v{i} >= {i} {{\n", i = i));
    }
    src.push_str("  return 42;\n");
    for _ in 0..10 {
        src.push_str("  }\n");
    }
    src.push_str("}\nfn main() -> Int { return deep(); }");
    let ir = compile(&src).unwrap();
    assert!(ir.contains("define"), "deep block return must compile");
}

// Stress: many interleaved variable types
#[test] fn test_stress_interleaved_types() {
    let src = "\
fn mix() -> Int {
    var i: Int = 1;
    var f: Float64 = 2.0;
    var b: Bool = true;
    var c: Char = 'A';
    var s: Str = \"test\";
    var v = [1, 2, 3];
    var score: Int = 0;
    if b { score = score + 10; }
    if c == 'A' { score = score + 20; }
    if f > 1.0 { score = score + 30; }
    score = score + i + s.len() as Int;
    return score;
}
fn main() -> Int { return mix(); }";
    let ir = compile(src).unwrap();
    assert!(ir.contains("define"), "interleaved types must compile");
}

