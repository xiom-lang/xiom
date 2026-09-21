// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32(4000000000) as UInt64 preserves value
fn main() -> Int {
  var a: UInt32 = 4000000000;
  var b: UInt64 = a as UInt64;
  if b == 4000000000 as UInt64 { return 0; }
  return 1;
}
