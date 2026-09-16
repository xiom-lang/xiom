// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Narrow int in if condition
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = 127 as Int8;
  if a < b && a == -128 as Int8 && b == 127 as Int8 { return 0; }
  return 1;
}
