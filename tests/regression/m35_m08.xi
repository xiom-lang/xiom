// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M08: is-prime (trial division)
fn is_prime(n: Int) -> Int {
  if n <= 1 { return 0; }
  if n <= 3 { return 1; }
  if n % 2 == 0 { return 0; }
  if n % 3 == 0 { return 0; }
  var i: Int = 5;
  while i * i <= n {
    if n % i == 0 { return 0; }
    if n % (i + 2) == 0 { return 0; }
    i = i + 6;
  }
  return 1;
}
fn main() -> Int {
  if is_prime(2) == 1 && is_prime(3) == 1 && is_prime(4) == 0 && is_prime(17) == 1 && is_prime(25) == 0 && is_prime(97) == 1 { return 0; }
  return 1;
}
