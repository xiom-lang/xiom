// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z06: Float64 += -- basic compound add
fn main() -> Int {
  var f: Float64 = 3.5;
  f += 2.5;
  if f == 6.0 { return 0; }
  return 1;
}
