// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8(128) as Int32 then negated should not exist in unsigned
fn main() -> Int {
  var a: UInt8 = 128;
  var b: Int32 = a as Int32;
  if b == 128 as Int32 { return 0; }
  return 1;
}
