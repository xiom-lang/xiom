// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Zero extend Int8 to Int32 (positive values)
fn main() -> Int {
  var a: Int8 = 127;
  var b: Int32 = a as Int32;
  if b == 127 as Int32 { return 0; }
  return 1;
}
