// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Mixed Float64 -> Int32 -> Int16 narrowing chain
fn main() -> Int {
  var f: Float64 = 32767.0;
  var i32: Int32 = f as Int32;
  var i16: Int16 = i32 as Int16;
  if i16 == 32767 { return 0; }
  return 1;
}
