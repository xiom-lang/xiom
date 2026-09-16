// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8(255) as Int16 then as Int64: chain must preserve 255, not -1
fn main() -> Int {
  var a: UInt8 = 255;
  var b: Int16 = a as Int16;
  var c: Int64 = b as Int64;
  if c == 255 as Int64 { return 0; }
  return 1;
}
