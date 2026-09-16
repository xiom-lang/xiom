// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 arithmetic shift right sign fills
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = 8;
  var c: Int16 = a >> b;
  if c == -128 as Int16 { return 0; }
  return 1;
}
