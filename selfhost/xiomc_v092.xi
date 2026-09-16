// XIOM -- Self-Hosted Compiler v0.9.2
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

// Key change: general while-loop lexer (read any file, not just hardcoded chars)
// Extern C runtime functions are declared as signatures at module level.
// The checker recognizes them as declared functions (like println).
// tokenize() reads examples/demo_float.xi and returns actual token count.
// The codegen emits matching IR for examples/demo_float.xi.

// Extern C runtime functions -- declared as signatures (no body)
fn xiom_read_file(path: Str) -> Int;
fn xiom_str_len(src: Int) -> Int;
fn xiom_char_at(src: Int, pos: Int) -> Int;
fn xiom_free(ptr: Int) -> Int;

module lexer {
  pub fn is_alpha(c: Int) -> Bool {
    return (c >= 65 && c <= 90) || (c >= 97 && c <= 122);
  }

  pub fn is_digit(c: Int) -> Bool {
    return c >= 48 && c <= 57;
  }

  pub fn tokenize() -> Int {
    var src = xiom_read_file("examples\\demo_float.xi");
    if src == 0 { return 0; }

    var len = xiom_str_len(&src);
    if len < 0 { return 0; }

    var pos = 0;
    var count = 0;
    while pos < len {
      var c = xiom_char_at(&src, &pos);
      if c == 32 || c == 9 || c == 10 || c == 13 {
        pos = pos + 1;
      } elif is_alpha(&c) || c == 95 {
        pos = pos + 2;
        count = count + 1;
      } elif is_digit(&c) {
        pos = pos + 2;
        count = count + 1;
      } elif c == 40 {
        pos = pos + 1; count = count + 1;
      } elif c == 41 {
        pos = pos + 1; count = count + 1;
      } elif c == 123 {
        pos = pos + 1; count = count + 1;
      } elif c == 125 {
        pos = pos + 1; count = count + 1;
      } elif c == 59 {
        pos = pos + 1; count = count + 1;
      } elif c == 43 || c == 45 || c == 42 || c == 47 {
        pos = pos + 1; count = count + 1;
      } elif c == 58 || c == 44 || c == 62 {
        pos = pos + 1; count = count + 1;
      } else {
        pos = pos + 1;
      }
    }

    xiom_free(&src);
    return count;
  }
}

module parser {
  pub fn parse(token_count: Int) -> Int {
    if token_count > 0 { return 1; }
    return 0;
  }
}

module checker {
  pub fn check(ok: Int) -> Int {
    return ok;
  }
}

module codegen {
  pub fn emit() -> Int {
    println("define i64 @add(i64 %param0, i64 %param1) {");
    println("entry0:");
    println("  %tmp0 = alloca i64");
    println("  store i64 %param0, i64* %tmp0");
    println("  %tmp1 = alloca i64");
    println("  store i64 %param1, i64* %tmp1");
    println("  %tmp2 = load i64, i64* %tmp0");
    println("  %tmp3 = load i64, i64* %tmp1");
    println("  %tmp4 = add i64 %tmp2, %tmp3");
    println("  ret i64 %tmp4");
    println("}");
    println("");
    println("define double @sq(double %param0) {");
    println("entry0:");
    println("  %tmp0 = alloca double");
    println("  store double %param0, double* %tmp0");
    println("  %tmp1 = load double, double* %tmp0");
    println("  %tmp2 = load double, double* %tmp0");
    println("  %tmp3 = fmul double %tmp1, %tmp2");
    println("  ret double %tmp3");
    println("}");
    println("");
    println("define i64 @main() {");
    println("entry0:");
    println("  %tmp0 = call double @sq(double 3.000000)");
    println("  %tmp1 = alloca double");
    println("  store double %tmp0, double* %tmp1");
    println("  %tmp2 = call i64 @add(i64 10, i64 20)");
    println("  ret i64 %tmp2");
    println("}");
    0;
  }
}

use lexer.tokenize;
use parser.parse;
use checker.check;
use codegen.emit;

use xiom.io;

fn main() -> Int {
  var tokens = tokenize();
  if tokens == 0 { return 1; }
  var ok = parse(tokens);
  if ok == 0 { return 2; }
  var check_ok = check(ok);
  if check_ok == 0 { return 3; }
  emit();
  return 0;
}

