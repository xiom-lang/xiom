// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 arithmetic shift right (-2147483648 >> 31 = -1)
fn main() -> Int {
  var a: Int32 = -2147483648 as Int32;
  var b: Int32 = 31;
  var c: Int32 = a >> b;
  if c == -1 as Int32 { return 0; }
  return 1;
}
