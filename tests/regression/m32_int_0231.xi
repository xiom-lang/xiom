// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 shift left by 0 (identity)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = 0;
  var c: Int8 = a << b;
  if c == a { return 0; }
  return 1;
}
