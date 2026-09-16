// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: All four int types in elif chain
fn main() -> Int {
  var a: Int8 = 50;
  var b: Int16 = a as Int16;
  var c: Int32 = b as Int32;
  var d: Int64 = c as Int64;
  var sum: Int64 = d + d + d + d;
  if sum == 200 as Int64 {
    return 0;
  } elif sum == 100 as Int64 {
    return 1;
  } elif sum == 50 as Int64 {
    return 2;
  } elif sum == 25 as Int64 {
    return 3;
  } else {
    return 4;
  }
  return 5;
}
