// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Unary negation on Int
fn main() -> Int {
  var a: Int = 42;
  var b: Int = -a;
  var c: Int = -b;
  if b == -42 && c == 42 {
    return 0;
  }
  return 1;
}
