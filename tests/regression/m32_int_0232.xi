// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 shift left by type width (1 << 8 wraps/zeros)
fn main() -> Int {
  var a: Int8 = 1;
  var b: Int8 = 8;
  var c: Int8 = a << b;
  if c == 0 as Int8 || c == 1 as Int8 { return 0; }
  return 1;
}
