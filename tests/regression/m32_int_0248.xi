// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 neg of zero
fn main() -> Int {
  var a: Int8 = 0;
  var b: Int8 = -a;
  if b == 0 as Int8 { return 0; }
  return 1;
}
