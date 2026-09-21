// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Downcast Int64 -> Int32 -> Int16 -> Int8 (safe values)
fn main() -> Int {
  var a: Int64 = 42;
  var b: Int32 = a as Int32;
  var c: Int16 = b as Int16;
  var d: Int8 = c as Int8;
  if d == 42 as Int8 {
    return 0;
  }
  return 1;
}
