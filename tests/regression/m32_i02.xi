// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 max value and edge with elif
fn main() -> Int {
  var a: Int16 = 32767;
  var b: Int16 = -32768;
  var zero: Int16 = 0;
  if a == 32767 as Int16 {
    if b < zero {
      if b + 32768 == 0 as Int16 {
        return 0;
      }
      return 1;
    }
    return 2;
  } elif b > a {
    return 3;
  } else {
    return 4;
  }
  return 5;
}
