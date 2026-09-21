// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 modulo
fn main() -> Int {
  var a: Int16 = 30000;
  var b: Int16 = 7;
  var mod_val: Int16 = a % b;
  if mod_val == 5 as Int16 {
    return 0;
  }
  return 1;
}
