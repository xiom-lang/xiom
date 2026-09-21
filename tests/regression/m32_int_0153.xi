// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 add wraparound (255 + 1 = 0)
fn main() -> Int {
  var a: UInt8 = 255;
  var b: UInt8 = 1;
  var c: UInt8 = a + b;
  if c == 0 as UInt8 { return 0; }
  return 1;
}
