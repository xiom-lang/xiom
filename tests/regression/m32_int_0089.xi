// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Logical NOT on integer comparison results
fn main() -> Int {
  var a: Int = 5;
  var b: Int = 10;
  if !(a > b) && !(a == b) && (a < b) {
    return 0;
  }
  return 1;
}
