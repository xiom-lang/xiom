// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 max + UInt8 max = 254 (no overflow for this specific add)
fn main() -> Int {
  var a: UInt8 = 127;
  var b: UInt8 = 127;
  var c: UInt8 = a + b;
  if c == 254 as UInt8 { return 0; }
  return 1;
}
