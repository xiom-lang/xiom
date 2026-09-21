// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 bitwise XOR self-clear
fn main() -> Int {
  var a: Int32 = -2147483648 as Int32;
  var b: Int32 = a ^ a;
  if b == 0 as Int32 { return 0; }
  return 1;
}
