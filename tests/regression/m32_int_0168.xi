// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 bitwise XOR
fn main() -> Int {
  var a: UInt16 = 65535;
  var b: UInt16 = a ^ a;
  if b == 0 as UInt16 { return 0; }
  return 1;
}
