// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 cast from Int32 (truncation)
fn main() -> Int {
  var a: Int32 = 70000;
  var b: Int16 = a as Int16;
  if b == 4464 as Int16 { return 0; }
  return 1;
}
