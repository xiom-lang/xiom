// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: All integer type widths cross-cast to Int
fn main() -> Int {
  var i8: Int8 = -1 as Int8;
  var i16: Int16 = -1 as Int16;
  var i32: Int32 = -1 as Int32;
  var i64: Int64 = -1 as Int64;
  var u8: UInt8 = 1;
  var u16: UInt16 = 1;
  var u32: UInt32 = 1;
  var u64: UInt64 = 1;
  var r: Int = i8 as Int + i16 as Int + i32 as Int + i64 as Int + u8 as Int + u16 as Int + u32 as Int + u64 as Int;
  if r == -4 + 4 { return 0; }
  return 1;
}
