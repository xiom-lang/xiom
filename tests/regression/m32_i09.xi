// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 bitwise AND/OR/XOR with elif
fn main() -> Int {
  var x: Int8 = 0x0F;
  var y: Int8 = 0x3C;
  var and_val: Int8 = x & y;
  var or_val: Int8 = x | y;
  var xor_val: Int8 = x ^ y;
  if and_val == 12 as Int8 {
    if or_val == 63 as Int8 {
      if xor_val == 51 as Int8 {
        return 0;
      } elif xor_val == 0 as Int8 {
        return 1;
      }
      return 2;
    } elif or_val == 0 as Int8 {
      return 3;
    } else {
      return 4;
    }
  } elif and_val == 0 as Int8 {
    return 5;
  } else {
    return 6;
  }
  return 7;
}
