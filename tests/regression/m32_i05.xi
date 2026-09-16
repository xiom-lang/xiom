// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 cast to Int16 arithmetic with elif
fn main() -> Int {
  var x: Int8 = 100;
  var y: Int16 = x as Int16;
  var z: Int16 = y * 200;
  if z == 20000 as Int16 {
    return 0;
  } elif z == 10000 as Int16 {
    return 1;
  } elif z == 0 as Int16 {
    return 2;
  } else {
    return 3;
  }
  return 4;
}
