// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Cross-cast Int8 -> UInt16 -> Int32 -> UInt64 chain
fn main() -> Int {
  var a: Int8 = -1;
  var b: UInt16 = a as UInt16;
  var c: Int32 = b as Int32;
  if c == 65535 as Int32 {
    return 0;
  }
  return 1;
}
