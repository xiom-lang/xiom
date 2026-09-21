// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 at min boundary (-128 - 1 = 127)
fn main() -> Int {
  var min: Int8 = -128 as Int8;
  var x: Int8 = min - 1 as Int8;
  if x == 127 as Int8 { return 0; }
  return 1;
}
