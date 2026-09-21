// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 multiple chained operations
fn main() -> Int {
  var a: Int8 = 64;
  var b: Int8 = a * 2 as Int8;
  var c: Int8 = b + 1 as Int8;
  var d: Int8 = c - 1 as Int8;
  if d == -128 as Int8 { return 0; }
  return 1;
}
