// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8(255) > UInt8(254) and 254 + 1 wraparound === 255
fn main() -> Int {
  var a: UInt8 = 254;
  var b: UInt8 = 1;
  var c: UInt8 = a + b;
  if c == 255 as UInt8 { return 0; }
  return 1;
}
