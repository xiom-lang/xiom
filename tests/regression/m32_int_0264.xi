// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8(128) as UInt16 then as Int32: chain must preserve 128
fn main() -> Int {
  var a: UInt8 = 128;
  var b: UInt16 = a as UInt16;
  var c: Int32 = b as Int32;
  if c == 128 as Int32 { return 0; }
  return 1;
}
