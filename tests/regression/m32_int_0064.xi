// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Bitwise NOT on Int
fn main() -> Int {
  var a: Int = 0;
  var result: Int = ~a;
  if result == -1 {
    return 0;
  }
  return 1;
}
