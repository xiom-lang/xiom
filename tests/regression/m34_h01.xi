// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H01: Int8->Int16->Int32->Int64 widening chain -- promotion preserves value
fn main() -> Int {
  var a: Int8 = 42;
  var b: Int16 = a as Int16;
  var c: Int32 = b as Int32;
  var d: Int64 = c as Int64;
  if a as Int64 == 42 && b == 42 as Int16 && c == 42 as Int32 && d == 42 as Int64 {
    return 0;
  }
  return 1;
}
