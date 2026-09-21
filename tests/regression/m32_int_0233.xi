// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 logical shift right (128 >> 7 = 1)
fn main() -> Int {
  var a: UInt8 = 128;
  var b: UInt8 = 7;
  var c: UInt8 = a >> b;
  if c == 1 as UInt8 { return 0; }
  return 1;
}
