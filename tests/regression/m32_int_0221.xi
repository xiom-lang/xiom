// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Boundary arithmetic with Int8 step-by-step overflow
fn main() -> Int {
  var a: Int8 = 126;
  var b: Int8 = a + 1 as Int8;
  var c: Int8 = b + 1 as Int8;
  if c == -128 as Int8 { return 0; }
  return 1;
}
