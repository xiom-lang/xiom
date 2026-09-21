// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16(-32768) -> UInt16(32768) -> Int16(-32768) roundtrip
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: UInt16 = a as UInt16;
  var c: Int16 = b as Int16;
  if c == a { return 0; }
  return 1;
}
