// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 maximum value 4294967295
fn main() -> Int {
  var x: UInt32 = 4294967295;
  var y: UInt32 = 4294967294;
  if x > y {
    return 0;
  }
  return 1;
}
