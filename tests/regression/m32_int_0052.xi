// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Division by zero guard -- Int16
fn main() -> Int {
  var a: Int16 = 100;
  var b: Int16 = 0;
  if b != 0 as Int16 {
    var r: Int16 = a / b;
    if r == 0 as Int16 { return 1; }
    return 1;
  }
  return 0;
}
