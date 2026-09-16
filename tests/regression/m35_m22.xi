// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M22: hailstone sequence -- verify max value reached in Collatz sequence
fn hailstone_max(n: Int) -> Int {
  if n <= 0 { return 0; }
  var max_val = n;
  var x = n;
  while x != 1 {
    if x % 2 == 0 { x = x / 2; }
    else { x = 3 * x + 1; }
    if x > max_val { max_val = x; }
  }
  return max_val;
}
fn main() -> Int {
  if hailstone_max(1) == 1 && hailstone_max(3) == 16 && hailstone_max(6) == 16 && hailstone_max(9) == 52 { return 0; }
  return 1;
}
