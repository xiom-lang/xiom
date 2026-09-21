// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z08: Float64 *= -- basic compound multiply
fn main() -> Int {
  var f: Float64 = 4.0;
  f *= 2.5;
  if f == 10.0 { return 0; }
  return 1;
}
