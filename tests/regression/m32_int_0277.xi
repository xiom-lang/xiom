// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Function taking UInt8 param widening to Int
fn widen8(a: UInt8) -> Int {
  return a as Int;
}
fn main() -> Int {
  var x: UInt8 = 200;
  var y: Int = widen8(x);
  if y == 200 { return 0; }
  return 1;
}
