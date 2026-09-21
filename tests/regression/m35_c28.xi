// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C28: early return from function -- return at multiple points based on guard conditions
fn safe_divide(a: Int, b: Int) -> Int {
  if b == 0 { return 0; }
  return a / b;
}
fn clamp(x: Int, lo: Int, hi: Int) -> Int {
  if x < lo { return lo; }
  if x > hi { return hi; }
  return x;
}
fn validate_positive(x: Int) -> Bool {
  if x <= 0 { return false; }
  if x > 1000 { return false; }
  return true;
}
fn main() -> Int {
  if safe_divide(10, 2) != 5 { return 1; }
  if safe_divide(10, 0) != 0 { return 2; }
  if clamp(-5, 0, 10) != 0 { return 3; }
  if clamp(15, 0, 10) != 10 { return 4; }
  if clamp(7, 0, 10) != 7 { return 5; }
  if validate_positive(5) != true { return 6; }
  if validate_positive(0) != false { return 7; }
  if validate_positive(2000) != false { return 8; }
  return 0;
}
