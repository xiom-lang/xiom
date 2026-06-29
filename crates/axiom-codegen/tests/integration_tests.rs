use axiom_lexer::Lexer;
use axiom_parser::Parser;
use axiom_codegen::IrEmitter;

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
    assert!(ir.contains("define i64 @make"));
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
