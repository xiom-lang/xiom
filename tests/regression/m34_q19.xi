// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Q19: Contract with module -- cross-module contract chain
module math_utils {
  pub fn abs(x: Int) -> Int
    ensures: result >= 0
  {
    if x < 0 { return -x; }
    return x;
  }
  pub fn gcd(a: Int, b: Int) -> Int
    requires: a >= 0
    requires: b >= 0
    ensures: result >= 0
  {
    if b == 0 { return a; }
    return gcd(b, a % b);
  }
}
module check {
  use math_utils.abs;
  use math_utils.gcd;
  pub fn safe_abs_chain(x: Int, y: Int) -> Int
    requires: y >= 0
    ensures: result >= 0
  {
    var a = abs(x);
    var g = gcd(a, y);
    return g;
  }
}
use check.safe_abs_chain;
fn main() -> Int {
  var r1 = safe_abs_chain(-24, 18);
  var r2 = safe_abs_chain(-30, 12);
  var r3 = safe_abs_chain(42, 7);
  if r1 == 6 && r2 == 6 && r3 == 7 { return 0; }
  return 1;
}
