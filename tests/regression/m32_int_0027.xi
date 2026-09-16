// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 modulo -- 2000000000 % 9999 = 20
fn main() -> Int {
  var a: Int32 = 2000000000;
  var b: Int32 = 9999;
  var mod_val: Int32 = a % b;
  if mod_val == 20 as Int32 {
    return 0;
  }
  return 1;
}
