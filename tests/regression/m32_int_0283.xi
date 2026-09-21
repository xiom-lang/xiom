// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 comparison with high value (128 > 0)
fn main() -> Int {
  var a: UInt8 = 128;
  var b: UInt8 = 0;
  if a > b { return 0; }
  return 1;
}
