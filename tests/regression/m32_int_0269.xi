// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 multiplication inside unsigned range (256 * 128 = 32768)
fn main() -> Int {
  var a: UInt16 = 256;
  var b: UInt16 = 128;
  var c: UInt16 = a * b;
  if c == 32768 as UInt16 { return 0; }
  return 1;
}
