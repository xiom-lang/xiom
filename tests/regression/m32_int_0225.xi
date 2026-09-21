// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 multiple chained operations
fn main() -> Int {
  var a: Int16 = 16384;
  var b: Int16 = a * 2 as Int16;
  var c: Int16 = b + 1 as Int16;
  if c == -32767 as Int16 { return 0; }
  return 1;
}
