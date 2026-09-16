// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Char value 255 cast to Int should be 255, not -1
fn main() -> Int {
  var c: Char = '\xff';
  var n: Int = c as Int;
  if n == 255 { return 0; }
  return 1;
}
