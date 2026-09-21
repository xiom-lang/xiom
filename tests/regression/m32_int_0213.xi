// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Function multiple narrow params
fn mix8(a: Int8, b: UInt8, c: Int16) -> Int16 {
  var s: Int8 = a + b as Int8;
  return s as Int16 + c;
}
fn main() -> Int {
  var r: Int16 = mix8(100, 27, 0);
  if r == 127 as Int16 { return 0; }
  return 1;
}
