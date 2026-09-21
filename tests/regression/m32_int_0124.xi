// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 sign extension from negative Int8
fn main() -> Int {
  var a: Int8 = -42 as Int8;
  var b: Int16 = a as Int16;
  if b == -42 as Int16 { return 0; }
  return 1;
}
