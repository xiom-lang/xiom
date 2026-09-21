// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X08: Combinatorial + Differential -- factorial loop vs recursion with generic+contract
fn fact_loop(n: Int) -> Int
  requires: n >= 0
  ensures: result >= 1
{
  var result: Int = 1;
  var i: Int = 1;
  while i <= n { result = result * i; i = i + 1; }
  return result;
}
fn fact_rec(n: Int) -> Int
  requires: n >= 0
  ensures: result >= 1
{
  if n <= 1 { return 1; }
  return n * fact_rec(n - 1);
}
enum FactMethod { Loop, Recursive }
fn factorial(m: FactMethod, n: Int) -> Int {
  match m { Loop => fact_loop(n), Recursive => fact_rec(n), }
}
fn main() -> Int {
  var a5 = factorial(FactMethod.Loop, 5);
  var b5 = factorial(FactMethod.Recursive, 5);
  var a0 = factorial(FactMethod.Loop, 0);
  var b0 = factorial(FactMethod.Recursive, 0);
  if a5 == b5 && a0 == b0 && a5 == 120 { return 0; }
  return 1;
}
