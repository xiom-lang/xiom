// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 bitwise NOT via XOR (65535 ^ 0xAAAA)
fn main() -> Int {
  var a: UInt16 = 43690;
  var mask: UInt16 = 65535;
  var b: UInt16 = a ^ mask;
  if b == 21845 as UInt16 { return 0; }
  return 1;
}
