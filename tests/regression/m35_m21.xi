// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M21: Collatz sequence length (steps to reach 1)
fn collatz_len(n: Int) -> Int {
  if n <= 0 { return 0; }
  var steps: Int = 0;
  var x = n;
  while x != 1 {
    if x % 2 == 0 { x = x / 2; }
    else { x = 3 * x + 1; }
    steps = steps + 1;
  }
  return steps;
}
fn main() -> Int {
  if collatz_len(1) == 0 && collatz_len(2) == 1 && collatz_len(3) == 7 && collatz_len(6) == 8 && collatz_len(27) == 111 { return 0; }
  return 1;
}
