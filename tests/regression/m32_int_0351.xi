// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 roundtrip UInt16 -> Int -> UInt16
fn main() -> Int {
  var a: UInt16 = 50000;
  var b: Int = a as Int;
  var c: UInt16 = b as UInt16;
  if c == a { return 0; }
  return 1;
}
