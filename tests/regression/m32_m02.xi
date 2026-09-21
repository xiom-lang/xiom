// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M02: pub fn visibility -- access pub fn via use import
module calc {
  pub fn mul(a: Int, b: Int) -> Int { return a * b; }
}
use calc.mul;
fn main() -> Int {
  if mul(6, 7) == 42 { return 0; }
  return 1;
}
