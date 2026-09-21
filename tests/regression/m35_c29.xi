// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C29: tail call pattern -- recursive call in tail position with accumulator
fn gcd(a: Int, b: Int) -> Int {
  if b == 0 { return a; }
  return gcd(b, a % b);
}
fn sum_recursive_acc(n: Int, acc: Int) -> Int {
  if n <= 0 { return acc; }
  return sum_recursive_acc(n - 1, acc + n);
}
fn fibonacci(n: Int, a: Int, b: Int) -> Int {
  if n <= 0 { return a; }
  return fibonacci(n - 1, b, a + b);
}
fn main() -> Int {
  if gcd(48, 18) != 6 { return 1; }
  if gcd(17, 13) != 1 { return 2; }
  if sum_recursive_acc(5, 0) != 15 { return 3; }
  if sum_recursive_acc(10, 0) != 55 { return 4; }
  if fibonacci(6, 0, 1) != 8 { return 5; }
  if fibonacci(10, 0, 1) != 55 { return 6; }
  return 0;
}
