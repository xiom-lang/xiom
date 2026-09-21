// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 cast from Int16 (truncation)
fn main() -> Int {
  var a: Int16 = 256;
  var b: UInt8 = a as UInt8;
  if b == 0 as UInt8 { return 0; }
  return 1;
}
