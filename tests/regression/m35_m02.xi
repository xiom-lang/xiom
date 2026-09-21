// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M02: absolute value
fn abs_val(n: Int) -> Int {
  if n < 0 { return 0 - n; }
  return n;
}
fn main() -> Int {
  if abs_val(0) == 0 && abs_val(42) == 42 && abs_val(-42) == 42 && abs_val(-1) == 1 { return 0; }
  return 1;
}
