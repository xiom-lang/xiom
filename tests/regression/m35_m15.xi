// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M15: sum of arithmetic series: n*(a1 + an)/2
fn arith_sum(a1: Int, d: Int, n: Int) -> Int {
  if n <= 0 { return 0; }
  var an = a1 + (n - 1) * d;
  return n * (a1 + an) / 2;
}
fn main() -> Int {
  if arith_sum(1, 1, 10) == 55 && arith_sum(2, 2, 5) == 30 && arith_sum(1, 0, 5) == 5 && arith_sum(0, 1, 1) == 0 { return 0; }
  return 1;
}
