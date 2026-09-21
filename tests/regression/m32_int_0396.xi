// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 to Float64 roundtrip (negative value)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var f: Float64 = a as Float64;
  var b: Int8 = f as Int8;
  if b == a { return 0; }
  return 1;
}
