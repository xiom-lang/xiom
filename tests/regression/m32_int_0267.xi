// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 addition that stays within unsigned range (100 + 100 = 200)
fn main() -> Int {
  var a: UInt8 = 100;
  var b: UInt8 = 100;
  var c: UInt8 = a + b;
  if c == 200 as UInt8 { return 0; }
  return 1;
}
