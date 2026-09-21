// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 cast to Int32 arithmetic with elif/else
fn main() -> Int {
  var a: Int16 = -10000;
  var b: Int32 = a as Int32;
  var c: Int32 = b * 3;
  var d: Int32 = c / 2;
  var e: Int32 = d + 5000;
  var neg_10k: Int32 = -10000;
  var neg_15k: Int32 = -15000;
  var zero_i32: Int32 = 0;
  if e == neg_10k {
    return 0;
  } elif e == neg_15k {
    return 1;
  } elif e == zero_i32 {
    return 2;
  } else {
    return 3;
  }
  return 4;
}
