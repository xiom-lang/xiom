// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 overflow sub (0 - 1 = 65535)
fn main() -> Int {
  var a: UInt16 = 0;
  var b: UInt16 = 1;
  var c: UInt16 = a - b;
  if c == 65535 as UInt16 { return 0; }
  return 1;
}
