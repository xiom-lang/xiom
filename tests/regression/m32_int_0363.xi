// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Two-level function with UInt16 widening
fn add16(a: UInt16, b: UInt16) -> UInt16 {
  return a + b;
}
fn triple16(x: UInt16) -> UInt16 {
  return add16(add16(x, x), x);
}
fn main() -> Int {
  var v: UInt16 = triple16(20000);
  if v == 60000 as UInt16 { return 0; }
  return 1;
}
