// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Final comprehensive: all types, casts, arithmetic, comparisons
fn add8(a: Int8, b: Int8) -> Int8 { return a + b; }
fn sub16(a: Int16, b: Int16) -> Int16 { return a - b; }
fn mul32(a: Int32, b: Int32) -> Int32 { return a * b; }
fn main() -> Int {
  var i8: Int8 = add8(64, 64);
  var u8: UInt8 = 255;
  var i16: Int16 = sub16(0, 1);
  var u16: UInt16 = i16 as UInt16;
  var i32: Int32 = mul32(2, 1073741824);
  var u32: UInt32 = i8 as UInt32 + 128 as UInt32;
  var ch: Char = u8 as Char;
  if i8 == -128 as Int8 {
    if i16 == -1 as Int16 {
      if u16 == 65535 as UInt16 {
        if ch as UInt8 == 255 as UInt8 { return 0; }
      }
    }
  }
  return 1;
}
