// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 near max multiplication (255 * 255 = 65025)
fn main() -> Int {
  var a: UInt16 = 255;
  var b: UInt16 = 255;
  var c: UInt16 = a * b;
  if c == 65025 as UInt16 { return 0; }
  return 1;
}
