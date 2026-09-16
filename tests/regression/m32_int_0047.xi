// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 arithmetic
fn main() -> Int {
  var a: UInt32 = 2000000000;
  var b: UInt32 = 1000000000;
  var sum: UInt32 = a + b;
  var sub: UInt32 = a - b;
  if sum == 3000000000 as UInt32 && sub == 1000000000 as UInt32 {
    return 0;
  }
  return 1;
}
