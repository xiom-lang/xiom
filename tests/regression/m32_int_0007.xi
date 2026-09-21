// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 modulo
fn main() -> Int {
  var a: Int8 = 100;
  var b: Int8 = 7;
  var mod_val: Int8 = a % b;
  if mod_val == 2 as Int8 {
    return 0;
  }
  return 1;
}
