// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8(-128) -> UInt8(128) -> Int8(-128) roundtrip
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: UInt8 = a as UInt8;
  var c: Int8 = b as Int8;
  if c == a { return 0; }
  return 1;
}
