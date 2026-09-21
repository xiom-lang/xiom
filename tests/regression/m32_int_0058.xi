// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Unsigned to signed conversion UInt8 -> Int8
fn main() -> Int {
  var a: UInt8 = 255;
  var b: Int8 = a as Int8;
  if b == -1 as Int8 {
    return 0;
  }
  return 1;
}
