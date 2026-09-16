// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A21: Collatz conjecture -- count steps to reach 1 for given n
fn collatz_steps(n: Int) -> Int {
  var x: Int = n;
  var c: Int = 0;
  while x != 1 {
    if x % 2 == 0 { x = x / 2; } else { x = 3 * x + 1; }
    c = c + 1;
  }
  return c;
}
fn main() -> Int {
  if collatz_steps(1) != 0 { return 1; }
  if collatz_steps(2) != 1 { return 2; }
  if collatz_steps(6) != 8 { return 3; }
  if collatz_steps(10) != 6 { return 4; }
  if collatz_steps(27) != 111 { return 5; }
  return 0;
}
