// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X21: Tail recursion -- deep recursive calls optimized for tail position
fn tail_sum(n: Int, acc: Int) -> Int {
  if n <= 0 { return acc; }
  return tail_sum(n - 1, acc + n);
}
fn tail_fact(n: Int, acc: Int) -> Int {
  if n <= 1 { return acc; }
  return tail_fact(n - 1, acc * n);
}
fn tail_pow(base: Int, exp: Int, acc: Int) -> Int {
  if exp <= 0 { return acc; }
  return tail_pow(base, exp - 1, acc * base);
}
fn tail_gcd(a: Int, b: Int) -> Int {
  if b == 0 { return a; }
  return tail_gcd(b, a - (a / b) * b);
}
fn main() -> Int {
  if tail_sum(100, 0) != 5050 { return 1; }
  if tail_sum(10, 0) != 55 { return 2; }
  if tail_fact(5, 1) != 120 { return 3; }
  if tail_fact(7, 1) != 5040 { return 4; }
  if tail_pow(2, 10, 1) != 1024 { return 5; }
  if tail_pow(3, 5, 1) != 243 { return 6; }
  if tail_gcd(48, 18) != 6 { return 7; }
  if tail_gcd(100, 25) != 25 { return 8; }
  return 0;
}
