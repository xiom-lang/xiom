// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 left and right shift with elif
fn main() -> Int {
  var a: Int32 = 1;
  var left: Int32 = a << 10;
  var right: Int32 = left >> 5;
  var or_shift: Int32 = left | right;
  if left == 1024 as Int32 {
    if right == 32 as Int32 {
      if or_shift == 1056 as Int32 {
        return 0;
      } elif or_shift == 1024 as Int32 {
        return 1;
      } else {
        return 2;
      }
    } elif right == 16 as Int32 {
      return 3;
    } else {
      return 4;
    }
  } elif left == 512 as Int32 {
    return 5;
  } else {
    return 6;
  }
  return 7;
}
