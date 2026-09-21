// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 overflow -- add wraps
fn main() -> Int {
  var a: UInt32 = 4294967295;
  var b: UInt32 = 1;
  var sum: UInt32 = a + b;
  if sum == 0 as UInt32 {
    return 0;
  }
  return 1;
}
