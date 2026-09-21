// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: UInt8 maximum value -- 255 must be representable
fn main() -> Int {
  var x: UInt8 = 255;
  if x == 255 as UInt8 {
    return 0;
  }
  return 1;
}
