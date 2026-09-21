// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Function returning UInt8 from computation with high value
fn compute8() -> UInt8 {
  var a: UInt8 = 200;
  var b: UInt8 = 55;
  return a + b;
}
fn main() -> Int {
  var x: UInt8 = compute8();
  if x == 255 as UInt8 { return 0; }
  return 1;
}
