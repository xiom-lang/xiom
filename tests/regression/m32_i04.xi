// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 large value arithmetic with elif
fn main() -> Int {
  var a: Int64 = 4000000000000;
  var b: Int64 = 2000000000000;
  var diff: Int64 = a - b;
  var quot: Int64 = a / b;
  if diff == 2000000000000 as Int64 {
    if quot == 2 as Int64 {
      return 0;
    } elif quot == 1 as Int64 {
      return 1;
    } else {
      return 2;
    }
  } elif diff < b {
    return 3;
  } else {
    return 4;
  }
  return 5;
}
