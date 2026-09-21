// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 overflow -- add near max wraps
fn main() -> Int {
  var a: Int32 = 2147483647;
  var b: Int32 = 1;
  var sum: Int32 = a + b;
  if sum == -2147483648 as Int32 {
    return 0;
  }
  return 1;
}
