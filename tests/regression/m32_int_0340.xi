// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 pow2 check: -128 is power of 2 in 8-bit
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = a & (a - 1 as Int8);
  if b == 0 as Int8 { return 0; }
  return 1;
}
