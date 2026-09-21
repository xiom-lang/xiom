// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 cast from Int8 negative (bitwise 255 = -1)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: UInt8 = a as UInt8;
  if b == 255 as UInt8 { return 0; }
  return 1;
}
