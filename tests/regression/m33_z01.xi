// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z01: Int += -- basic compound add
fn main() -> Int {
  var x: Int = 10;
  x += 5;
  if x == 15 { return 0; }
  return 1;
}
