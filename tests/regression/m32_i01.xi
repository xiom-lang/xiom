// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 range classification with elif
fn main() -> Int {
  var x: Int8 = 127;
  if x > 100 as Int8 {
    return 0;
  } elif x > 50 as Int8 {
    return 1;
  } elif x > 0 as Int8 {
    return 2;
  } else {
    return 3;
  }
  return 4;
}
