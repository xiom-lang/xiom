// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 shift left (1 << 15 = 32768)
fn main() -> Int {
  var a: UInt16 = 1;
  var b: UInt16 = 15;
  var c: UInt16 = a << b;
  if c == 32768 as UInt16 { return 0; }
  return 1;
}
