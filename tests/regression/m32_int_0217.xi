// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 at max boundary (127 + 1 = -128)
fn main() -> Int {
  var max: Int8 = 127;
  var x: Int8 = max + 1 as Int8;
  if x == -128 as Int8 { return 0; }
  return 1;
}
