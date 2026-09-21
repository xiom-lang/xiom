// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 addition with result > 32767 must stay correct
fn main() -> Int {
  var a: UInt16 = 30000;
  var b: UInt16 = 10000;
  var c: UInt16 = a + b;
  if c == 40000 as UInt16 { return 0; }
  return 1;
}
