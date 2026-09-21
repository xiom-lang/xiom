// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Combined bitwise and arithmetic
fn main() -> Int {
  var a: Int = 0xFF;
  var b: Int = 0x0F;
  var c: Int = (a & b) * 10 + (a | b) / 5;
  if c == 201 {
    return 0;
  }
  return 1;
}
