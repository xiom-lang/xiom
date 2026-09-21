// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 bitwise ops at boundaries
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = 32767 as Int16;
  var c: Int16 = a & b;
  if c == 0 as Int16 { return 0; }
  return 1;
}
