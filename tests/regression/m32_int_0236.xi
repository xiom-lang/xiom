// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Cast chain Int8 -> Int16 -> Int32 -> Int64 (sign extends)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: Int16 = a as Int16;
  var c: Int32 = b as Int32;
  var d: Int64 = c as Int64;
  if d == -1 as Int64 { return 0; }
  return 1;
}
