// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X06: Combinatorial + Differential -- power loop vs recursion with struct+contract
type PowInput = { base: Int; exp: Int; }
fn pow_loop(input: PowInput) -> Int
  requires: input.exp >= 0
  ensures: result >= 1
{
  var result: Int = 1;
  var i: Int = 0;
  while i < input.exp { result = result * input.base; i = i + 1; }
  return result;
}
fn pow_rec(b: Int, e: Int) -> Int
  requires: e >= 0
  ensures: result >= 1
{
  if e == 0 { return 1; }
  return b * pow_rec(b, e - 1);
}
enum Method { Iter, Recurse }
fn power(m: Method, base: Int, exp: Int) -> Int {
  match m {
    Iter => pow_loop(PowInput{ base: base; exp: exp; }),
    Recurse => pow_rec(base, exp),
  }
}
fn main() -> Int {
  var a = power(Method.Iter, 2, 8);
  var b = power(Method.Recurse, 2, 8);
  var c = power(Method.Iter, 5, 3);
  var d = power(Method.Recurse, 5, 3);
  if a == b && c == d && a == 256 && c == 125 { return 0; }
  return 1;
}
