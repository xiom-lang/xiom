// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Char comparisons
fn main() -> Int {
  var a: UInt8 = 'A' as UInt8;
  var b: UInt8 = 'Z' as UInt8;
  if a < b { return 0; }
  return 1;
}
