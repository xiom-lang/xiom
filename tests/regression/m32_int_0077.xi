// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Hex literal with arithmetic and shifts
fn main() -> Int {
  var a: Int = 0x10;
  var b: Int = 0x0F;
  var and_val: Int = a & b;
  var or_val: Int = a | b;
  var xor_val: Int = a ^ b;
  if and_val == 0 && or_val == 0x1F && xor_val == 0x1F {
    return 0;
  }
  return 1;
}
