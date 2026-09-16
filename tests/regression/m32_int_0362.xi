// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Two-level function calling with narrow int params
fn mul8(a: Int8, b: Int8) -> Int8 {
  return a * b;
}
fn square8(x: Int8) -> Int8 {
  return mul8(x, x);
}
fn main() -> Int {
  var v: Int8 = square8(16);
  if v == 0 as Int8 { return 0; }
  return 1;
}
