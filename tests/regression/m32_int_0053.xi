// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Upcast Int8 -> Int16 -> Int32 -> Int64
fn main() -> Int {
  var a: Int8 = 42;
  var b: Int16 = a as Int16;
  var c: Int32 = b as Int32;
  var d: Int64 = c as Int64;
  if d == 42 as Int64 {
    return 0;
  }
  return 1;
}
