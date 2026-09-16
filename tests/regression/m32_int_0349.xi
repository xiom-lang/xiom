// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Mixed multiply UInt8(128) * Int8(2) through Int
fn main() -> Int {
  var a: UInt8 = 128;
  var b: Int8 = 2;
  var p: Int = a as Int * b as Int;
  if p == 256 { return 0; }
  return 1;
}
