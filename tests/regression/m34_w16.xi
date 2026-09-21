// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W16: Unsigned right shift -- logical shift on UInt and UInt32
fn main() -> Int {
  var x: UInt = 0xF000;
  var x32: UInt32 = 0xF000 as UInt32;
  var r: UInt = x >> 4;
  var r32: UInt32 = x32 >> 4;
  var r2: UInt = x >> 8;
  var r2_32: UInt32 = x32 >> 8;
  if r == 0xF00 && r32 == 0xF00 as UInt32 &&
     r2 == 0xF0 && r2_32 == 0xF0 as UInt32 { return 0; }
  return 1;
}
