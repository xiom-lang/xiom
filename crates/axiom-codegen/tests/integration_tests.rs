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
