// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Signed to unsigned conversion Int8 -> UInt8
fn main() -> Int {
  var a: Int8 = -1;
  var b: UInt8 = a as UInt8;
  if b == 255 as UInt8 {
    return 0;
  }
  return 1;
}
