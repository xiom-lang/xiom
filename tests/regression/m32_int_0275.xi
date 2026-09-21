// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt32 division (3000000000 / 2 = 1500000000)
fn main() -> Int {
  var a: UInt32 = 3000000000;
  var b: UInt32 = 2;
  var c: UInt32 = a / b;
  if c == 1500000000 as UInt32 { return 0; }
  return 1;
}
