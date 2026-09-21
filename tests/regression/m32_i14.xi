// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Negative arithmetic across Int8/Int16/Int32/Int64 with elif
// Correct values: sum = -4295033920, half = -2147516960
fn main() -> Int {
  var a: Int8 = -64;
  var b: Int16 = -1024;
  var c: Int32 = -65536;
  var d: Int64 = -4294967296;
  var a64: Int64 = a as Int64;
  var b64: Int64 = b as Int64;
  var c64: Int64 = c as Int64;
  var sum: Int64 = a64 + b64 + c64 + d;
  var half: Int64 = sum / 2;
  var expected_half: Int64 = -2147516960;
  var expected_sum: Int64 = -4295033920;
  var zero64: Int64 = 0;
  if half == expected_half {
    if sum == expected_sum {
      return 0;
    } elif sum == zero64 {
      return 1;
    } else {
      return 2;
    }
  } elif half == zero64 {
    return 3;
  } elif half > zero64 {
    return 4;
  } else {
    return 5;
  }
  return 6;
}
