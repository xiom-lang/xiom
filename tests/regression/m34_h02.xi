// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H02: Int64->Int32->Int16->Int8 narrowing chain -- truncation of upper bits
// 258 (0x102) truncates to 2 in Int8: upper byte discarded
fn main() -> Int {
  var a: Int64 = 258;
  var b: Int32 = a as Int32;
  var c: Int16 = b as Int16;
  var d: Int8 = c as Int8;
  if b == 258 as Int32 && d == 2 as Int8 { return 0; }
  return 1;
}
