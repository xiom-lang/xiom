// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Cast chain Int64 -> Int32 -> Int16 -> Int8 (truncates)
fn main() -> Int {
  var a: Int64 = 257;
  var b: Int32 = a as Int32;
  var c: Int16 = b as Int16;
  var d: Int8 = c as Int8;
  if d == 1 as Int8 { return 0; }
  return 1;
}
