// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 comparison: 50000 < 60000 must be true
fn main() -> Int {
  var a: UInt16 = 50000;
  var b: UInt16 = 60000;
  if a < b && b > a { return 0; }
  return 1;
}
