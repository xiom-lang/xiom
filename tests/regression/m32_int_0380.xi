// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 sign extension: -128 >> 7 = -1, then +1 = 0
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = a >> 7;
  var c: Int8 = b + 1 as Int8;
  if c == 0 as Int8 { return 0; }
  return 1;
}
