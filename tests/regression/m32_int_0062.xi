// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Bitwise OR on Int
fn main() -> Int {
  var a: Int = 0x0F;
  var b: Int = 0xF0;
  var result: Int = a | b;
  if result == 0xFF {
    return 0;
  }
  return 1;
}
