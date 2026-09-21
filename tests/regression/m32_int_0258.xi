// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16(40000) as Int should be 40000, not -25536 (zext vs sext)
fn main() -> Int {
  var a: UInt16 = 40000;
  var b: Int = a as Int;
  if b == 40000 { return 0; }
  return 1;
}
