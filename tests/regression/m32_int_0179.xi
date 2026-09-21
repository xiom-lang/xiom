// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 comparisons
fn main() -> Int {
  var a: UInt32 = 0;
  var b: UInt32 = 4294967295;
  if a < b && b > 0 as UInt32 { return 0; }
  return 1;
}
