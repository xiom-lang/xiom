// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt64 bitwise NOT via XOR
fn main() -> Int {
  var a: UInt64 = 0;
  var b: UInt64 = 18446744073709551615;
  var c: UInt64 = a ^ b;
  if c == b { return 0; }
  return 1;
}
