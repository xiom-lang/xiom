// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 bitwise at max
fn main() -> Int {
  var a: UInt32 = 4294967295;
  var b: UInt32 = a | a;
  if b == a { return 0; }
  return 1;
}
