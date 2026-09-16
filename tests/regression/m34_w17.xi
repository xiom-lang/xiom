// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-W17: Hex literal masking -- 0xFF & x extracts low byte across types
fn main() -> Int {
  var a: Int = 0x12345678;
  var b: Int32 = 0x5678 as Int32;
  var c: UInt = 0xABCDEF99;
  var lo_a: Int = a & 0xFF;
  var lo_b: Int32 = b & (0xFF as Int32);
  var lo_c: UInt = c & (0xFF as UInt);
  var mid_a: Int = (a >> 8) & 0xFF;
  var hi_a: Int = (a >> 24) & 0xFF;
  if lo_a == 0x78 && lo_b == 0x78 as Int32 && lo_c == 0x99 &&
     mid_a == 0x56 && hi_a == 0x12 { return 0; }
  return 1;
}
