// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 min (-128) value through arithmetic chain preserved
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = a + 0 as Int8;
  var c: Int8 = b - 0 as Int8;
  var d: Int8 = c * 1 as Int8;
  if d == -128 as Int8 { return 0; }
  return 1;
}
