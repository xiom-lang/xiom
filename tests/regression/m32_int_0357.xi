// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 0xAAAA (43690) widening to Int32 through sext vs zext
fn main() -> Int {
  var a: UInt16 = 43690;
  var b: Int32 = a as Int32;
  if b == 43690 as Int32 { return 0; }
  return 1;
}
