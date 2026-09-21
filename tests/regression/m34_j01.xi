// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J01: Multiple modules in one file -- basic cross-module calls
module math_a {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
}
module math_b {
  pub fn mul(a: Int, b: Int) -> Int { return a * b; }
}
module math_c {
  pub fn neg(a: Int) -> Int { return 0 - a; }
}
use math_a.add;
use math_b.mul;
use math_c.neg;
fn main() -> Int {
  var r1 = add(3, 4);
  var r2 = mul(5, 6);
  var r3 = neg(10);
  if r1 == 7 && r2 == 30 && r3 == -10 { return 0; }
  return 1;
}
