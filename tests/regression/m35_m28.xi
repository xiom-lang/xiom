// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M28: ceil division (a + b - 1) / b for positive integers
fn ceil_div(a: Int, b: Int) -> Int {
  if b <= 0 { return 0; }
  return (a + b - 1) / b;
}
fn main() -> Int {
  if ceil_div(5, 2) == 3 && ceil_div(10, 3) == 4 && ceil_div(1, 5) == 1 && ceil_div(10, 10) == 1 && ceil_div(0, 7) == 0 { return 0; }
  return 1;
}
