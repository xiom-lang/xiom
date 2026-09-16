// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 add wraparound (127 + 1 = -128)
fn main() -> Int {
  var a: Int8 = 127;
  var b: Int8 = 1;
  var c: Int8 = a + b;
  if c == -128 as Int8 { return 0; }
  return 1;
}
