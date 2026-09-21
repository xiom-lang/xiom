// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A13: LCM -- compute via formula lcm(a,b) = a * b / gcd(a,b)
fn gcd(m: Int, n: Int) -> Int {
  var x: Int = m;
  var y: Int = n;
  while y != 0 {
    var t: Int = y;
    y = x % y;
    x = t;
  }
  return x;
}
fn lcm(a: Int, b: Int) -> Int {
  var g: Int = gcd(a, b);
  return a / g * b;
}
fn main() -> Int {
  if lcm(4, 6) != 12 { return 1; }
  if lcm(12, 18) != 36 { return 2; }
  if lcm(7, 13) != 91 { return 3; }
  if lcm(1, 100) != 100 { return 4; }
  return 0;
}
