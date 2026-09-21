// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 cast from Int64 (truncation)
fn main() -> Int {
  var a: Int64 = 4294967296;
  var b: Int32 = a as Int32;
  if b == 0 as Int32 { return 0; }
  return 1;
}
