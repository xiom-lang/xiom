// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X13: Combinatorial + Differential -- contract check vs manual check with generic+enum
fn safe_div(a: Int, b: Int) -> Int
  requires: b != 0
  ensures: result * b == a
{
  return a / b;
}
fn manual_div(a: Int, b: Int) -> Int {
  if b != 0 { return a / b; }
  return 0;
}
enum DivMethod { Contract, Manual }
fn divide(m: DivMethod, a: Int, b: Int) -> Int {
  match m {
    Contract => safe_div(a, b),
    Manual => manual_div(a, b),
  }
}
fn main() -> Int {
  var r1 = divide(DivMethod.Contract, 100, 4);
  var r2 = divide(DivMethod.Manual, 100, 4);
  var r3 = divide(DivMethod.Contract, 63, 7);
  var r4 = divide(DivMethod.Manual, 63, 7);
  if r1 == r2 && r3 == r4 && r1 == 25 && r3 == 9 { return 0; }
  return 1;
}
