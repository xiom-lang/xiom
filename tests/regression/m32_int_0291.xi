// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Truncation UInt16(257) to UInt8 yields 1
fn main() -> Int {
  var a: UInt16 = 257;
  var b: UInt8 = a as UInt8;
  if b == 1 as UInt8 { return 0; }
  return 1;
}
