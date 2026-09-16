// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M24: Deep recursion -- tail-recursive sum
fn sum_to(n: Int, acc: Int) -> Int {
  if n == 0 { return acc; }
  return sum_to(n - 1, n + acc);
}
// Mutual recursion: even/odd test
fn is_even(n: Int) -> Bool {
  if n == 0 { return true; }
  return is_odd(n - 1);
}
fn is_odd(n: Int) -> Bool {
  if n == 0 { return false; }
  return is_even(n - 1);
}
fn main() -> Int {
  var s: Int = sum_to(100, 0);
  if s == 5050 && is_even(20) && is_odd(21) { return 0; }
  return 1;
}
