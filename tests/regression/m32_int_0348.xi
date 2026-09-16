// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Mixed adding UInt16(40000) + Int16(10000) through Int
fn main() -> Int {
  var a: UInt16 = 40000;
  var b: Int16 = 10000;
  var ax: Int = a as Int;
  var bx: Int = b as Int;
  if ax + bx == 50000 { return 0; }
  return 1;
}
