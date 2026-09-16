// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X12: --no-contracts patterns -- works both with and without contract checks
fn is_positive(x: Int) -> Bool {
  return x > 0;
}
fn abs_val(x: Int) -> Int {
  if x < 0 { return 0 - x; }
  return x;
}
fn clamp_val(x: Int, lo: Int, hi: Int) -> Int {
  var v = x;
  if v < lo { v = lo; }
  if v > hi { v = hi; }
  return v;
}
fn safe_range(x: Int, lo: Int, hi: Int) -> Bool {
  return x >= lo && x <= hi;
}
fn compute_with_asserts(a: Int, b: Int) -> Int {
  var sum = a + b;
  if sum < 0 { return 0; }
  return sum;
}
fn main() -> Int {
  if !is_positive(5) { return 1; }
  if is_positive(-1) { return 2; }
  if abs_val(-10) != 10 { return 3; }
  if abs_val(7) != 7 { return 4; }
  if clamp_val(5, 0, 10) != 5 { return 5; }
  if clamp_val(-2, 0, 10) != 0 { return 6; }
  if clamp_val(15, 0, 10) != 10 { return 7; }
  if !safe_range(5, 0, 10) { return 8; }
  if safe_range(-1, 0, 10) { return 9; }
  if compute_with_asserts(10, 20) != 30 { return 10; }
  if compute_with_asserts(-50, 10) != 0 { return 11; }
  return 0;
}
