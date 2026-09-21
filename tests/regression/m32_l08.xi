// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-L08: Compound assignment in loop -- +=, -=, *=, /= inside while
fn main() -> Int {
  var x: Int = 1;
  var i: Int = 0;
  while i < 4 {
    x += 3;
    i += 1;
  }
  // x: 1->4->7->10->13
  x -= 5;
  // x: 13-5 = 8
  x *= 3;
  // x: 8*3 = 24
  x /= 8;
  // x: 24/8 = 3
  if x == 3 { return 0; }
  return 1;
}
