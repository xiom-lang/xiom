// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 mod with negative (-128 % 3 = -2)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = 3;
  var c: Int8 = a % b;
  if c == -2 as Int8 { return 0; }
  return 1;
}
