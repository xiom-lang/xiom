// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z03: Int *= -- basic compound multiply
fn main() -> Int {
  var x: Int = 6;
  x *= 7;
  if x == 42 { return 0; }
  return 1;
}
