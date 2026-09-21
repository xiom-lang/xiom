// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Hex literal operations
fn main() -> Int {
  var a: Int = 0xDEAD;
  var b: Int = 0xBEEF;
  var sum: Int = a + b;
  if sum == 0x19D9C {
    return 0;
  }
  return 1;
}
