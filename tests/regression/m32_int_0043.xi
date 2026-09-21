// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 overflow -- add wraps
fn main() -> Int {
  var a: UInt8 = 255;
  var b: UInt8 = 1;
  var sum: UInt8 = a + b;
  if sum == 0 as UInt8 {
    return 0;
  }
  return 1;
}
