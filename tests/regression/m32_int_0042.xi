// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 zero and arithmetic
fn main() -> Int {
  var a: UInt8 = 100;
  var b: UInt8 = 50;
  var sum: UInt8 = a + b;
  var sub: UInt8 = a - b;
  if sum == 150 as UInt8 && sub == 50 as UInt8 {
    return 0;
  }
  return 1;
}
