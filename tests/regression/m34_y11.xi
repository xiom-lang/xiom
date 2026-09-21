// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y11: while loop factorial + contract + Option + enum + module + recursion
enum Strategy { Loop, Recursion }
fn fact_loop(n: Int) -> Option[Int] {
  var acc = 1;
  var i = 1;
  while i <= n { acc = acc * i; i = i + 1; }
  return Some(acc);
}
fn fact_rec(n: Int) -> Option[Int] {
  if n == 0 { return Some(1); }
  match fact_rec(n - 1) {
    Some(v) => Some(v * n),
    None => None,
  }
}
fn factorial[T](n: Int, s: Strategy) -> Option[Int]
  requires: n >= 0
{
  match s {
    Loop => fact_loop(n),
    Recursion => fact_rec(n),
  }
}
module math_utils {
  pub fn fact(n: Int, s: Strategy) -> Option[Int] { return factorial(n, s); }
  pub fn just_n(n: Int) -> Int { return n; }
}
use math_utils.fact;
use math_utils.just_n;
fn main() -> Int {
  match fact(5, Strategy.Loop) {
    Some(v) => { if v != 120 { return 1; } }
    None => { return 2; }
  }
  match fact(5, Strategy.Recursion) {
    Some(v) => { if v != 120 { return 3; } }
    None => { return 4; }
  }
  return 0;
}
