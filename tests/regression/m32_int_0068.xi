// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Bitwise OR/XOR/NOT on UInt16
fn main() -> Int {
  var a: UInt16 = 0xFF00;
  var b: UInt16 = 0x00FF;
  var or_val: UInt16 = a | b;
  var and_val: UInt16 = a & b;
  var xor_val: UInt16 = a ^ b;
  var not_a: UInt16 = ~a;
  if or_val == 0xFFFF as UInt16 && and_val == 0 as UInt16 && xor_val == 0xFFFF as UInt16 {
    return 0;
  }
  return 1;
}
