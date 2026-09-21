// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z05: Int %= -- basic compound modulo
fn main() -> Int {
  var x: Int = 17;
  x %= 5;
  if x == 2 { return 0; }
  return 1;
}
