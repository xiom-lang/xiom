// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M29: Compound assignment operators -- verify runtime behavior
fn main() -> Int {
  var x: Int = 10;
  var y: Int = 20;
  x += 5;
  y -= 5;
  x *= 2;
  y /= 3;
  x %= 7;
  if x == 2 && y == 5 { return 0; }
  return 1;
}
