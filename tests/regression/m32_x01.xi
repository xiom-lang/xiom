// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X01: Combinatorial + Differential -- while vs recursive sum with generic+contract
fn sum_while(n: Int) -> Int
  requires: n >= 0
  ensures: result >= 0
{
  var total: Int = 0;
  var i: Int = 1;
  while i <= n { total = total + i; i = i + 1; }
  return total;
}
fn sum_rec(n: Int) -> Int
  requires: n >= 0
  ensures: result >= 0
{
  if n == 0 { return 0; }
  return n + sum_rec(n - 1);
}
fn main() -> Int {
  var a = sum_while(50);
  var b = sum_rec(50);
  if a == b && a == 1275 { return 0; }
  return 1;
}
