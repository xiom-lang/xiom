// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 max (127) value through arithmetic chain preserved
fn main() -> Int {
  var a: Int8 = 127;
  var b: Int8 = a * 1 as Int8;
  var c: Int8 = b - 0 as Int8;
  if c == 127 { return 0; }
  return 1;
}
