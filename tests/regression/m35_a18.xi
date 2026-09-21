// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A18: Power function -- compute a^b via iterative multiplication
fn pow(base: Int, exp: Int) -> Int {
  if exp == 0 { return 1; }
  var r: Int = 1;
  var e: Int = exp;
  while e > 0 {
    r = r * base;
    e = e - 1;
  }
  return r;
}
fn main() -> Int {
  if pow(2, 0) != 1 { return 1; }
  if pow(2, 5) != 32 { return 2; }
  if pow(3, 4) != 81 { return 3; }
  if pow(5, 3) != 125 { return 4; }
  if pow(10, 6) != 1000000 { return 5; }
  if pow(7, 1) != 7 { return 6; }
  return 0;
}
