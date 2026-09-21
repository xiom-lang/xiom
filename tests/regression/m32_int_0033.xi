// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 addition
fn main() -> Int {
  var a: Int64 = 1000000000000;
  var b: Int64 = 2000000000000;
  var sum: Int64 = a + b;
  if sum == 3000000000000 {
    return 0;
  }
  return 1;
}
