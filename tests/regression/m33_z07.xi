// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z07: Float64 -= -- basic compound subtract
fn main() -> Int {
  var f: Float64 = 10.0;
  f -= 3.5;
  if f == 6.5 { return 0; }
  return 1;
}
