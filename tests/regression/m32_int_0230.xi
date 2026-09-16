// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 mod by -1 edge
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = -1 as Int8;
  var c: Int8 = a % b;
  if c == 0 as Int8 { return 0; }
  return 1;
}
