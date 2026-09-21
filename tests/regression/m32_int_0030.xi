// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 negative operations
fn main() -> Int {
  var a: Int32 = -1000000;
  var b: Int32 = 500000;
  var sum: Int32 = a + b;
  var sub: Int32 = a - b;
  var mul: Int32 = a * 2;
  var div: Int32 = a / 3;
  if sum == -500000 as Int32 && sub == -1500000 as Int32 && mul == -2000000 as Int32 && div == -333333 as Int32 {
    return 0;
  }
  return 1;
}
