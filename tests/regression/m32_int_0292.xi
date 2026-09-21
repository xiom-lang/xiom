// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Truncation UInt32(65537) to UInt16 yields 1
fn main() -> Int {
  var a: UInt32 = 65537;
  var b: UInt16 = a as UInt16;
  if b == 1 as UInt16 { return 0; }
  return 1;
}
