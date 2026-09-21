// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 right shift (128 >> 1 = 64, logical)
fn main() -> Int {
  var a: UInt8 = 128;
  var b: UInt8 = 1;
  var c: UInt8 = a >> b;
  if c == 64 as UInt8 { return 0; }
  return 1;
}
