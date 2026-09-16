// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Multiple unsigned params through function chain
fn add_unsigned(a: UInt8, b: UInt16) -> Int {
  var x: Int = a as Int;
  var y: Int = b as Int;
  return x + y;
}
fn main() -> Int {
  var r: Int = add_unsigned(200, 50000);
  if r == 50200 { return 0; }
  return 1;
}
