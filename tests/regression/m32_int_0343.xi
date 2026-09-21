// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 narrowing to Int16 (truncation, sign bit of low 16 bits)
fn main() -> Int {
  var a: Int64 = 32767;
  var b: Int16 = a as Int16;
  if b == 32767 { return 0; }
  return 1;
}
