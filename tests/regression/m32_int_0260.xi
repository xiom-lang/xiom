// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32(3000000000) as Int should preserve value magnitude (zext)
fn main() -> Int {
  var a: UInt32 = 3000000000;
  var b: Int = a as Int;
  if b == 3000000000 { return 0; }
  return 1;
}
