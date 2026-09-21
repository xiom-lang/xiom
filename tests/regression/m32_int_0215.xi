// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Function return UInt8
fn get_max_u8() -> UInt8 {
  return 255;
}
fn main() -> Int {
  var x: UInt8 = get_max_u8();
  var y: UInt8 = x + 1 as UInt8;
  if y == 0 as UInt8 { return 0; }
  return 1;
}
