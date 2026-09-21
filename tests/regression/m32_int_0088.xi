// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Shift by zero -- identity operation
fn main() -> Int {
  var a: Int = 12345;
  var b: Int = a << 0;
  var c: Int = a >> 0;
  if b == 12345 && c == 12345 {
    return 0;
  }
  return 1;
}
