// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z04: Int /= -- basic compound divide
fn main() -> Int {
  var x: Int = 100;
  x /= 4;
  if x == 25 { return 0; }
  return 1;
}
