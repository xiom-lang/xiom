// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Comprehensive narrow int mix: casting, arithmetic, comparison, function
fn negate8(x: Int8) -> Int8 { return -x; }
fn add16(a: Int16, b: Int16) -> Int16 { return a + b; }
fn main() -> Int {
  var i8: Int8 = -128 as Int8;
  var u8: UInt8 = i8 as UInt8;
  var i16: Int16 = i8 as Int16;
  var u16: UInt16 = u8 as UInt16;
  var neg: Int8 = negate8(i8);
  var sum: Int16 = add16(i16, 1);
  if u8 == 128 as UInt8 {
    if u16 == 128 as UInt16 {
      if neg == i8 {
        if sum == -127 as Int16 { return 0; }
      }
    }
  }
  return 1;
}
