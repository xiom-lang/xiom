// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: Bitwise operations on Int
fn main() -> Int {
  var a: Int = 0xFF;
  var b: Int = 0x0F;
  var and_val: Int = a & b;
  var or_val: Int = a | b;
  var xor_val: Int = a ^ b;
  var shift_left: Int = 1 << 4;
  var shift_right: Int = 64 >> 3;
  if and_val == 15 && or_val == 255 && xor_val == 240 && shift_left == 16 && shift_right == 8 {
    return 0;
  }
  return 1;
}
