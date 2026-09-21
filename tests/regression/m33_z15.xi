// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z15: Chain multiple compound operations
fn main() -> Int {
  var a: Int = 2;
  var b: Int = 10;
  var c: Int = 50;
  a += 3;
  b -= 4;
  c *= 2;
  a *= b;
  c /= 5;
  a += c;
  if a == 50 && b == 6 && c == 20 { return 0; }
  return 1;
}
