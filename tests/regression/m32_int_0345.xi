// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 all bits set (255) widened through chain to Int
fn main() -> Int {
  var a: UInt8 = 255;
  var b: UInt16 = a as UInt16;
  var c: UInt32 = b as UInt32;
  var d: Int = c as Int;
  if d == 255 { return 0; }
  return 1;
}
