// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16(50000) as UInt32 preserves value
fn main() -> Int {
  var a: UInt16 = 50000;
  var b: UInt32 = a as UInt32;
  if b == 50000 as UInt32 { return 0; }
  return 1;
}
