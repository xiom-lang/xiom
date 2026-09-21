// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y09: multi-contract + while + enum + compound assign + module
enum OpMode { Sum, Product, Max }
fn sum_loop(n: Int) -> Int {
  var i = 1; var s = 0;
  while i <= n { s = s + i; i = i + 1; }
  return s;
}
fn prod_loop(n: Int) -> Int {
  var i = 1; var p = 1;
  while i <= n { p = p * i; i = i + 1; }
  return p;
}
fn max_val(a: Int, b: Int) -> Int { if a > b { return a; } return b; }
fn crunch[T](n: Int, mode: OpMode) -> Int
  requires: n >= 0
  requires: n <= 20
  ensures: result >= 0
{
  match mode {
    Sum => sum_loop(n),
    Product => prod_loop(n),
    Max => max_val(n, 10),
  }
}
module crunch_mod {
  pub fn process(n: Int, m: OpMode) -> Int { return crunch(n, m); }
  pub fn just_n(n: Int) -> Int { return n; }
}
use crunch_mod.process;
use crunch_mod.just_n;
fn main() -> Int {
  var r1 = process(4, OpMode.Sum);
  var r2 = process(4, OpMode.Product);
  var r3 = process(15, OpMode.Max);
  if r1 == 10 && r2 == 24 && r3 == 15 { return 0; }
  return 1;
}
