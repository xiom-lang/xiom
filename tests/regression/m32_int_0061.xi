// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Bitwise AND on Int
fn main() -> Int {
  var a: Int = 0xFF;
  var b: Int = 0xF0;
  var result: Int = a & b;
  if result == 0xF0 {
    return 0;
  }
  return 1;
}
