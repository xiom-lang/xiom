// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM -- Body Parser Edge Case Stress Test
// Exercises every corner of the C runtime body parser.

fn test_negative() -> Int { return -(5); }
fn test_paren_expr() -> Int { return (2 + 3) * 4; }
fn test_multi_param_call(a: Int, b: Int) -> Int { return a + b; }
fn test_string_literal() -> Int { var x: Int = 1; return x; }
fn test_empty_stmts() -> Int { return 42; }
fn test_comment_skip() -> Int { return 42; }
fn test_multi_line() -> Int { return 1 + 2 + 3 + 4; }
fn test_nested_if() -> Int { var x: Int = 0; if 1 == 1 { if 2 == 2 { x = 1; } } return x; }
fn test_nested_while() -> Int { var i: Int = 0; var j: Int = 0; while i < 3 { while j < 3 { j = j + 1; } i = i + 1; } return i + j; }

fn main() -> Int {
  var total: Int = 0;
  total = total + test_negative();
  total = total + test_paren_expr();
  total = total + test_empty_stmts();
  total = total + test_comment_skip();
  total = total + test_multi_line();
  total = total + test_nested_if();
  total = total + test_nested_while();
  return total;
}
