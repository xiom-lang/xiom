// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 maximum value 2147483647
fn main() -> Int {
  var x: Int32 = 2147483647;
  var y: Int32 = 2147483646;
  if x > y && x == 2147483647 as Int32 {
    return 0;
  }
  return 1;
}
