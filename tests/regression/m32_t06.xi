// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T06: Int to Char -- as cast from Int to Char
fn main() -> Int {
  var a: Char = 65 as Char;
  var b: Char = 66 as Char;
  if a == 'A' && b == 'B' { return 0; }
  return 1;
}
