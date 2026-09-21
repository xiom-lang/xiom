// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 subtraction
fn main() -> Int {
  var a: Int16 = 5000;
  var b: Int16 = 10000;
  var sub: Int16 = a - b;
  if sub == -5000 as Int16 {
    return 0;
  }
  return 1;
}
