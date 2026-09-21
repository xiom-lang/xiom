// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16(65535) as Int64 must be 65535, not -1 (zext vs sext)
fn main() -> Int {
  var a: UInt16 = 65535;
  var b: Int64 = a as Int64;
  if b == 65535 as Int64 { return 0; }
  return 1;
}
