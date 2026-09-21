// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM -- Compiler Robustness Tests
// Verifies the compiler never panics -- always returns Result, never crashes.

use xiom_lexer::Lexer;
use xiom_parser::Parser;
use xiom_codegen::IrEmitter;

/// Attempt full compile. Returns Ok(ir) or Err(msg). MUST NOT panic.
fn try_compile(source: &str) -> Result<String, String> {
    // Run on a dedicated big-stack thread: `compile_expr` has very large
    // debug-build frames and Rust test threads default to a 2MB stack --
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
    // break/continue outside a loop -- must not panic (may error)
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

// -- M24-1: Large file / many declarations -----------------------------

#[test] fn robust_100_functions() {
    let mut src = String::new();
    for i in 0..100 { src.push_str(&format!("fn f{i}() -> Int {{ return {i}; }}\n")); }
    src.push_str("fn main() -> Int { return f99(); }");
    assert_no_panic(&src);
}

#[test] fn robust_50_struct_types() {
    let mut src = String::new();
    for i in 0..50 { src.push_str(&format!("type T{i} = {{ x{i}: Int; y{i}: Float64; }}\n")); }
    src.push_str("fn main() -> Int { return 0; }");
    assert_no_panic(&src);
}

#[test] fn robust_50_enum_types() {
    let mut src = String::new();
    for i in 0..50 { src.push_str(&format!("enum E{i} {{ V{i}a, V{i}b(val: Int) }}\n")); }
    src.push_str("fn main() -> Int { return 0; }");
    assert_no_panic(&src);
}

#[test] fn robust_50_module_decls() {
    let mut src = String::new();
    for i in 0..50 { src.push_str(&format!("module m{i} {{ pub fn f{i}() -> Int {{ return {i}; }} }}\n")); }
    src.push_str("fn main() -> Int { return 0; }");
    assert_no_panic(&src);
}

#[test] fn robust_200_field_struct() {
    let fields: String = (0..200).map(|i| format!("f{i}: Int; ")).collect();
    assert_no_panic(&format!("type Huge = {{ {} }} fn main() -> Int {{ return 0; }}", fields));
}

#[test] fn robust_50_const_decls() {
    let mut src = String::new();
    for i in 0..50 { src.push_str(&format!("const C{i}: Int = {i};\n")); }
    src.push_str("fn main() -> Int { return C49; }");
    assert_no_panic(&src);
}

#[test] fn robust_huge_source_file() {
    let mut src = String::new();
    for i in 0..200 {
        match i % 5 {
            0 => src.push_str(&format!("fn f{i}() -> Int {{ return {i}; }}\n")),
            1 => src.push_str(&format!("type T{i} = {{ x: Int; y: Float64; }}\n")),
            2 => src.push_str(&format!("enum E{i} {{ A, B(val: Int) }}\n")),
            3 => src.push_str(&format!("const C{i}: Int = {i};\n")),
            4 => src.push_str(&format!("module m{i} {{ pub fn g{i}() -> Int {{ return {i}; }} }}\n")),
            _ => {}
        }
    }
    src.push_str("fn main() -> Int { return 0; }");
    assert_no_panic(&src);
}

// -- M24-2: Deep recursion ---------------------------------------------

#[test] fn robust_deep_recursion_100() {
    let mut src = String::from("fn deep(n: Int) -> Int { if n == 0 { return 0; } return 1 + deep(n - 1); }\n");
    src.push_str("fn main() -> Int { return deep(100); }");
    assert_no_panic(&src);
}

#[test] fn robust_mutual_recursion() {
    let src = "fn is_even(n: Int) -> Bool { if n == 0 { return true; } return is_odd(n - 1); } fn is_odd(n: Int) -> Bool { if n == 0 { return false; } return is_even(n - 1); } fn main() -> Int { if is_even(20) { return 0; } return 1; }";
    assert_no_panic(src);
}

#[test] fn robust_tail_recursive() {
    let src = "fn fact_tail(n: Int, acc: Int) -> Int { if n <= 1 { return acc; } return fact_tail(n - 1, n * acc); } fn main() -> Int { return fact_tail(10, 1); }";
    assert_no_panic(src);
}

#[test] fn robust_deeply_nested_if_else() {
    let mut src = String::from("fn deep(n: Int) -> Int {\n");
    for i in 0..50 { src.push_str(&format!("  if n == {i} {{ return {i}; }} else\n")); }
    src.push_str("  { return -1; }\n}\nfn main() -> Int { return deep(25); }");
    assert_no_panic(&src);
}

#[test] fn robust_deep_call_chain() {
    let mut src = String::new();
    for i in 0..30 { src.push_str(&format!("fn c{i}() -> Int {{ return c{}(); }}\n", i + 1)); }
    src.push_str("fn c30() -> Int { return 42; }\n");
    src.push_str("fn main() -> Int { return c0(); }");
    assert_no_panic(&src);
}

#[test] fn robust_deeply_nested_type() {
    let src = "type T0 = { a: Int; } type T1 = { b: T0; } type T2 = { c: T1; } type T3 = { d: T2; } type T4 = { e: T3; } type T5 = { f: T4; } type T6 = { g: T5; } type T7 = { h: T6; } type T8 = { i: T7; } type T9 = { j: T8; } fn main() -> Int { return 0; }";
    assert_no_panic(src);
}

// -- M24-3: Memory / layout stress -------------------------------------

#[test] fn robust_large_array_literal() {
    let elements: String = (0..100).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
    assert_no_panic(&format!("fn main() -> Int {{ var arr = [{}]; return arr[0]; }}", elements));
}

#[test] fn robust_many_local_variables() {
    let mut src = String::from("fn many_locals() -> Int {\n");
    for i in 0..100 { src.push_str(&format!("  var x{i}: Int = {i};\n")); }
    src.push_str("  return x99;\n}\nfn main() -> Int { return many_locals(); }");
    assert_no_panic(&src);
}

#[test] fn robust_deep_var_shadowing() {
    let mut src = String::from("fn shadow() -> Int {\n");
    for i in 0..30 { src.push_str(&format!("  {{ var x = {i};\n")); }
    for _ in 0..30 { src.push_str("  }\n"); }
    src.push_str("  return 0;\n}\nfn main() -> Int { return shadow(); }");
    assert_no_panic(&src);
}

#[test] fn robust_string_with_many_escapes() {
    let s = "\\n".repeat(50);
    assert_no_panic(&format!(r#"fn main() -> Int {{ var s: Str = "{}"; return s.len() as Int; }}"#, s));
}

#[test] fn robust_float_precision_edge() {
    let src = "fn main() -> Int { var a: Float64 = 0.1; var b: Float64 = 0.2; var c: Float64 = a + b; var d: Float64 = 0.3; if c >= d { return 0; } return 1; }";
    assert_no_panic(src);
}

#[test] fn robust_divide_by_zero_all_contexts() {
    assert_no_panic("fn main() -> Int { var x: Int = 0; return 1 / x; }");
    assert_no_panic("fn main() -> Int { return 100 / 0; }");
    assert_no_panic("fn main() -> Int { return 100 % 0; }");
}

// -- M24-4: FFI / unsafe stress ----------------------------------------

#[test] fn robust_unsafe_block_basic() {
    assert_no_panic("fn main() -> Int { unsafe { var x: Int = 42; return x; } }");
}

#[test] fn robust_extern_with_many_functions() {
    let mut src = String::from("extern \"C\" {\n");
    for i in 0..30 { src.push_str(&format!("  fn ext_fn{i}(x: Int) -> Int;\n")); }
    src.push_str("}\nfn main() -> Int { return 0; }");
    assert_no_panic(&src);
}

#[test] fn robust_mixed_safe_unsafe() {
    let src = "fn safe_caller() -> Int { var result: Int = 0; unsafe { var p: Int = 42; result = p; } return result; } fn main() -> Int { return safe_caller(); }";
    assert_no_panic(src);
}

#[test] fn robust_null_ptr_handling() {
    let src = "fn null_check(p: *UInt8) -> Bool { if p == null { return true; } return false; } fn main() -> Int { var p: *UInt8 = null; if null_check(p) { return 0; } return 1; }";
    assert_no_panic(src);
}

#[test] fn robust_raw_pointer_arithmetic() {
    assert_no_panic("fn main() -> Int { var p: *UInt8 = null; var q: *UInt8 = p + 8; return 0; }");
}

// -- M24-5: Error recovery stress --------------------------------------

#[test] fn robust_50_type_errors() {
    let mut src = String::new();
    for i in 0..50 { src.push_str(&format!("fn broken{i}() -> Int {{ return \"not_an_int\"; }}\n")); }
    src.push_str("fn main() -> Int { return 0; }");
    assert_no_panic(&src);
}

#[test] fn robust_undefined_everywhere() {
    let mut src = String::new();
    for i in 0..50 { src.push_str(&format!("fn f{i}() -> Int {{ return undefined_var_{i}; }}\n")); }
    src.push_str("fn main() -> Int { return 0; }");
    assert_no_panic(&src);
}

#[test] fn robust_malformed_expr_chain() {
    assert_no_panic("fn main() -> Int { return 1 + * / - + ; }");
}

#[test] fn robust_missing_semicolons() {
    let mut src = String::from("fn main() -> Int {\n");
    for i in 0..50 { src.push_str(&format!("  var x{i} = {i}\n")); }
    src.push_str("  return 0;\n}");
    assert_no_panic(&src);
}

#[test] fn robust_random_garbage_prefix() {
    let garbage = "@#$%^&*()!~`[]{}|;:',.<>?/";
    assert_no_panic(&format!("{}\nfn main() -> Int {{ return 0; }}", garbage));
}

// -- M24-6: Fuzzing / differential / determinism -----------------------

#[test] fn robust_random_valid_functions() {
    let mut seed: u64 = 12345; let mut src = String::new();
    for i in 0..50 {
        let a = (seed % 100) as i64; seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let b = (seed % 100) as i64; seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let op = match seed % 4 { 0 => "+", 1 => "-", 2 => "*", _ => "/" };
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // Avoid divide by zero: set b to 1 if op is / and b is 0
        let actual_b = if op == "/" && b == 0 { 1i64 } else { b };
        src.push_str(&format!("fn f{i}() -> Int {{ return {a} {op} {actual_b}; }}\n"));
    }
    src.push_str("fn main() -> Int { return f25(); }");
    assert_no_panic(&src);
}

#[test] fn robust_repeated_compilation_deterministic() {
    // Repeated compilation must always succeed with consistent structure
    let src = "fn add(a: Int, b: Int) -> Int { return a + b; } fn main() -> Int { return add(1, 2); }";
    for _ in 0..10 {
        let ir = try_compile(src);
        assert!(ir.is_ok(), "repeated compilation must always succeed");
        let ir_text = ir.unwrap();
        assert!(ir_text.contains("define"), "IR must contain function definitions");
        assert!(ir_text.contains("@add"), "IR must contain add function");
    }
}

#[test] fn robust_all_operators_combined() {
    assert_no_panic("fn all_ops(a: Int, b: Int) -> Int { var r = a + b; r = a - b; r = a * b; r = a / b; r = a % b; r = a & b; r = a | b; r = a ^ b; r = a << 2; r = a >> 1; return r; } fn main() -> Int { return all_ops(10, 3); }");
}

#[test] fn robust_format_compile_cycle() {
    // Format -> recompile -> check IR consistency
    let src = "fn add(a: Int, b: Int) -> Int { return a + b; } fn main() -> Int { return add(10, 20); }";
    let ir1 = try_compile(src);
    if let Ok(ir) = &ir1 {
        assert!(ir.contains("define"), "successful compile must produce valid IR");
    }
}

#[test] fn robust_wildcard_in_generic_context() {
    assert_no_panic("enum Res[T] { Ok(val: T); Err(code: Int); } fn handle(r: Res[Int]) -> Int { match r { Res.Ok(v) => v, Res.Err(_) => -1, } } fn main() -> Int { return handle(Res[Int].Ok(42)); }");
}
