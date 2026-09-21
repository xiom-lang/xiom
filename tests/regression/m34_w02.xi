// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W02: OR zero -- x | 0 == x on unsigned integer types
fn main() -> Int {
  var a: UInt = 99;
  var a8: UInt8 = 99 as UInt8;
  var a16: UInt16 = 99 as UInt16;
  var a32: UInt32 = 99 as UInt32;
  var b: UInt = a | (0 as UInt);
  var b8: UInt8 = a8 | (0 as UInt8);
  var b16: UInt16 = a16 | (0 as UInt16);
  var b32: UInt32 = a32 | (0 as UInt32);
  if b == 99 && b8 == 99 as UInt8 && b16 == 99 as UInt16 && b32 == 99 as UInt32 { return 0; }
  return 1;
}
