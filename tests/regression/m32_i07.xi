// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 cast to Int64 with elif branching
fn main() -> Int {
  var x: Int32 = 2000000000;
  var y: Int64 = x as Int64;
  var z: Int64 = y * 2;
  var w: Int64 = z - 1000000000;
  if w == 3000000000 as Int64 {
    return 0;
  } elif w == 2000000000 as Int64 {
    return 1;
  } elif w < 0 as Int64 {
    return 2;
  } else {
    return 3;
  }
  return 4;
}
