// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A25: Digit sum recursive -- compute sum of digits via recursion
fn digit_sum_rec(n: Int) -> Int {
  if n == 0 { return 0; }
  return n % 10 + digit_sum_rec(n / 10);
}
fn main() -> Int {
  if digit_sum_rec(0) != 0 { return 1; }
  if digit_sum_rec(5) != 5 { return 2; }
  if digit_sum_rec(123) != 6 { return 3; }
  if digit_sum_rec(9999) != 36 { return 4; }
  if digit_sum_rec(1000001) != 2 { return 5; }
  return 0;
}
