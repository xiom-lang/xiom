// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 left shift beyond width (1 << 16 = 0 or 1 depending on mod)
fn main() -> Int {
  var a: UInt16 = 1;
  var b: UInt16 = 16;
  var c: UInt16 = a << b;
  if c == 0 as UInt16 || c == 1 as UInt16 { return 0; }
  return 1;
}
