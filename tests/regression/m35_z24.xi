// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z24: recursion+generic+Option+Result+match+contract+module+enum+struct+compound_assign+while
type FibState = { prev: Int; curr: Int; n: Int; }
enum FibMode { Recursive, Iterative, Memo }
fn fib_rec(n: Int) -> Int {
  if n <= 1 { return n; }
  return fib_rec(n - 1) + fib_rec(n - 2);
}
fn fib_safe[T](n: Int, mode: FibMode) -> Result[Int, Str]
  requires: n >= 0
{
  if n > 30 { return Err("too big"); }
  match mode {
    Recursive => Ok(fib_rec(n)),
    Iterative => {
      if n <= 1 { return Ok(n); }
      var a = 0;
      var b = 1;
      var i = 1;
      while i < n { var t = b; b = a + b; a = t; i += 1; }
      Ok(b)
    }
    Memo => {
      if n <= 1 { return Ok(n); }
      var p = 0;
      var c = 1;
      var i = 2;
      while i <= n {
        var t = c;
        c = p + c;
        p = t;
        i += 1;
      }
      Ok(c);
    }
  }
}
module fib {
  pub fn compute(n: Int, m: FibMode) -> Result[Int, Str] { return fib_safe(n, m); }
  pub fn quick(n: Int) -> Result[Int, Str] { return fib_safe(n, FibMode.Iterative); }
}
use fib.compute;
use fib.quick;
fn main() -> Int {
  var chk = 0;
  match compute(10, FibMode.Recursive) { Ok(v) => { if v == 55 { chk += 1; } } Err(_) => {} }
  match compute(10, FibMode.Iterative) { Ok(v) => { if v == 55 { chk += 1; } } Err(_) => {} }
  match compute(10, FibMode.Memo) { Ok(v) => { if v == 55 { chk += 1; } } Err(_) => {} }
  if chk == 3 { return 0; }
  return 1;
}
