// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 addition
fn main() -> Int {
  var a: Int16 = 10000;
  var b: Int16 = 20000;
  var sum: Int16 = a + b;
  if sum == 30000 as Int16 {
    return 0;
  }
  return 1;
}
