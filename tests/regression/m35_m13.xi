// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M13: perfect number check (sum of proper divisors equals n)
fn is_perfect(n: Int) -> Int {
  if n <= 1 { return 0; }
  var sum: Int = 1;
  var i: Int = 2;
  while i * i <= n {
    if n % i == 0 {
      sum = sum + i;
      var pair = n / i;
      if pair != i { sum = sum + pair; }
    }
    i = i + 1;
  }
  if sum == n { return 1; }
  return 0;
}
fn main() -> Int {
  if is_perfect(6) == 1 && is_perfect(28) == 1 && is_perfect(496) == 1 && is_perfect(12) == 0 && is_perfect(8) == 0 { return 0; }
  return 1;
}
