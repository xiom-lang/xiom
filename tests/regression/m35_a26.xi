// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A26: Exponent by squaring -- fast binary exponentiation a^b % m
fn pow_mod(base: Int, exp: Int, m: Int) -> Int {
  var r: Int = 1;
  var b: Int = base % m;
  var e: Int = exp;
  while e > 0 {
    if e % 2 == 1 { r = (r * b) % m; }
    b = (b * b) % m;
    e = e / 2;
  }
  return r;
}
fn pow_exp(base: Int, exp: Int) -> Int {
  var r: Int = 1;
  var b: Int = base;
  var e: Int = exp;
  while e > 0 {
    if e % 2 == 1 { r = r * b; }
    b = b * b;
    e = e / 2;
  }
  return r;
}
fn main() -> Int {
  if pow_exp(2, 0) != 1 { return 1; }
  if pow_exp(2, 10) != 1024 { return 2; }
  if pow_exp(3, 5) != 243 { return 3; }
  if pow_exp(5, 3) != 125 { return 4; }
  if pow_mod(2, 10, 1000) != 24 { return 5; }
  return 0;
}
