// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 arithmetic shift right: -32768 >> 15 = -1
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = a >> 15;
  if b == -1 as Int16 { return 0; }
  return 1;
}
