// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8(64) * Int8(4) = 0 (wraps twice: 64*2=128=-128, 64*4=0)
fn main() -> Int {
  var a: Int8 = 64;
  var b: Int8 = 4;
  var c: Int8 = a * b;
  if c == 0 as Int8 { return 0; }
  return 1;
}
