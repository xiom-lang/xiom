// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Q03: Contract + generic -- generic fn with requires and ensures
fn max[T](a: T, b: T) -> T
  requires: a >= 0
  requires: b >= 0
  ensures: result >= a
  ensures: result >= b
{
  if a > b { return a; }
  return b;
}
fn bound[T](x: T) -> T
  requires: x >= 0
  ensures: result >= x
{
  return x + x;
}
fn main() -> Int {
  var r1: Int = max(7, 3);
  var r2: Int = bound(5);
  var r3: Int = max(r2, r1);
  if r1 == 7 && r2 == 10 && r3 == 10 { return 0; }
  return 1;
}
