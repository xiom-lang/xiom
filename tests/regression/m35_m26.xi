// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M26: is-power-of-2 check: n > 0 && (n & (n-1)) == 0
fn is_power_of_2(n: Int) -> Int {
  if n <= 0 { return 0; }
  if (n & (n - 1)) == 0 { return 1; }
  return 0;
}
fn main() -> Int {
  if is_power_of_2(1) == 1 && is_power_of_2(2) == 1 && is_power_of_2(3) == 0 && is_power_of_2(64) == 1 && is_power_of_2(100) == 0 { return 0; }
  return 1;
}
