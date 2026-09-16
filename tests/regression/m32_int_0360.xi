// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 max (255) value through arithmetic identity chain
fn main() -> Int {
  var a: UInt8 = 255;
  var b: UInt8 = a + 0 as UInt8;
  var c: UInt8 = b * 1 as UInt8;
  if c == 255 as UInt8 { return 0; }
  return 1;
}
