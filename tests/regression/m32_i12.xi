// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 negative modulo and division with elif/else
fn main() -> Int {
  var a: Int64 = -5000000000000;
  var b: Int64 = 3000000000000;
  var div: Int64 = a / b;
  var mod_val: Int64 = a % b;
  if div == -1 as Int64 {
    if mod_val == -2000000000000 as Int64 {
      return 0;
    } elif mod_val == 2000000000000 as Int64 {
      return 1;
    } else {
      return 2;
    }
  } elif div == -2 as Int64 {
    return 3;
  } elif div == 0 as Int64 {
    return 4;
  } else {
    return 5;
  }
  return 6;
}
