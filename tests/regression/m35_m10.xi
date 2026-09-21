// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M10: LCM via GCD
fn gcd(a: Int, b: Int) -> Int {
  var x = a;
  var y = b;
  while y != 0 {
    var t = y;
    y = x % y;
    x = t;
  }
  return x;
}
fn lcm(a: Int, b: Int) -> Int {
  if a == 0 || b == 0 { return 0; }
  return a / gcd(a, b) * b;
}
fn main() -> Int {
  if lcm(4, 6) == 12 && lcm(12, 18) == 36 && lcm(7, 13) == 91 && lcm(1, 1) == 1 && lcm(5, 10) == 10 { return 0; }
  return 1;
}
