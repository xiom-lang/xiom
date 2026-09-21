// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z19: Compound assignment near overflow edge -- Int8 near bounds
fn main() -> Int {
  var a: Int8 = 120;
  var b: Int8 = 5;
  a += b;
  if a == 125 as Int8 { return 0; }
  return 1;
}
