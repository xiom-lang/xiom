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
    assert!(ir.contains("declare i64 @axiom_is_sorted(i8*, i64)"));
    assert!(ir.contains("declare i64 @axiom_all(i8*, i64, i8*)"));
    assert!(ir.contains("declare i64 @axiom_none(i8*, i64, i8*)"));
    assert!(ir.contains("declare i64 @axiom_contains(i8*, i64)"));
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
