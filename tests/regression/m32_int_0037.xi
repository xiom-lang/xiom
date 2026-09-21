// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 modulo
fn main() -> Int {
  var a: Int64 = 3000000000007;
  var b: Int64 = 1000000;
  var mod_val: Int64 = a % b;
  if mod_val == 7 {
    return 0;
  }
  return 1;
}
