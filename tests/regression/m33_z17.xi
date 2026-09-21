// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z17: Compound assignment in function
fn accumulate(base: Int, a: Int, b: Int) -> Int {
  var x: Int = base;
  x += a;
  x *= b;
  x -= a;
  return x;
}
fn main() -> Int {
  var r: Int = accumulate(5, 3, 4);
  if r == 29 { return 0; }
  return 1;
}
