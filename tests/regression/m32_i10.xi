// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 multiply divide modulo with elif/else
fn main() -> Int {
  var a: Int16 = 1000;
  var b: Int16 = 7;
  var mul: Int16 = a * b;
  var div: Int16 = a / b;
  var mod_val: Int16 = a % b;
  if mul == 7000 as Int16 {
    if div == 142 as Int16 {
      if mod_val == 6 as Int16 {
        return 0;
      } elif mod_val == 0 as Int16 {
        return 1;
      } else {
        return 2;
      }
    } elif div == 143 as Int16 {
      return 3;
    } else {
      return 4;
    }
  } elif mul == 700 as Int16 {
    return 5;
  } else {
    return 6;
  }
  return 7;
}
