// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 bitwise OR with high bit set (32768 | 16384 = 49152)
fn main() -> Int {
  var a: UInt16 = 32768;
  var b: UInt16 = 16384;
  var c: UInt16 = a | b;
  if c == 49152 as UInt16 { return 0; }
  return 1;
}
