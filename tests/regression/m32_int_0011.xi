// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 minimum value -32768
fn main() -> Int {
  var x: Int16 = -32768;
  var y: Int16 = -32767;
  if x < y && x == -32768 as Int16 {
    return 0;
  }
  return 1;
}
