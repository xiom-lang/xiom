// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Q11: Contract with arithmetic overflow -- requires guards prevent overflow
fn safe_add(a: Int, b: Int) -> Int
  requires: a >= 0
  requires: b >= 0
  ensures: result >= a
  ensures: result >= b
{
  return a + b;
}
fn safe_mul(a: Int, b: Int) -> Int
  requires: a >= 0
  requires: b >= 0
  ensures: result >= a
{
  if a == 0 || b == 0 { return 0; }
  if b == 1 { return a; }
  var acc: Int = a;
  var i: Int = 1;
  while i < b { acc = safe_add(acc, a); i = safe_add(i, 1); }
  return acc;
}
fn compute(a: Int, b: Int, c: Int) -> Int
  requires: a >= 0
  requires: b >= 0
  requires: c >= 0
{
  var ab = safe_mul(a, b);
  return safe_add(ab, c);
}
fn main() -> Int {
  var r1 = safe_add(100, 200);
  var r2 = safe_mul(6, 7);
  var r3 = compute(4, 5, 10);
  if r1 == 300 && r2 == 42 && r3 == 30 { return 0; }
  return 1;
}
