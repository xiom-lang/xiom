// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M28: Correctness -- commutative and associative properties
fn add_ab(a: Int, b: Int) -> Int { return a + b; }
fn add_ba(a: Int, b: Int) -> Int { return b + a; }
fn mul_abc(a: Int, b: Int, c: Int) -> Int { return (a * b) * c; }
fn mul_acb(a: Int, b: Int, c: Int) -> Int { return (a * c) * b; }
fn main() -> Int {
  var r1 = add_ab(7, 3) == add_ba(7, 3);
  var r2 = mul_abc(2, 3, 5) == mul_acb(2, 3, 5);
  if r1 && r2 { return 0; }
  return 1;
}
