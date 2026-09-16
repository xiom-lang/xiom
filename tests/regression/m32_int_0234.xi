// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 arithmetic shift right sign fills (-8 >> 1 = -4)
fn main() -> Int {
  var a: Int8 = -8 as Int8;
  var b: Int8 = 1;
  var c: Int8 = a >> b;
  if c == -4 as Int8 { return 0; }
  return 1;
}
