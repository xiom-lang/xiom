// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Operator precedence: Int8 a + b * c
fn main() -> Int {
  var a: Int8 = 10;
  var b: Int8 = 5;
  var c: Int8 = 3;
  var r: Int8 = a + b * c;
  if r == 25 { return 0; }
  return 1;
}
