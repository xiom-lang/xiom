// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt64 mul wraparound (large * 2)
fn main() -> Int {
  var a: UInt64 = 9223372036854775808;
  var b: UInt64 = 2;
  var c: UInt64 = a * b;
  if c == 0 as UInt64 { return 0; }
  return 1;
}
