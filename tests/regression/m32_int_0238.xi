// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Cast chain UInt8 -> UInt16 -> UInt32 -> UInt64 (zero extends)
fn main() -> Int {
  var a: UInt8 = 255;
  var b: UInt16 = a as UInt16;
  var c: UInt32 = b as UInt32;
  var d: UInt64 = c as UInt64;
  if d == 255 as UInt64 { return 0; }
  return 1;
}
