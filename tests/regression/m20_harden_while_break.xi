// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn sum_even(n: Int) -> Int {
  var sum = 0;
  var i = 0;
  while i <= n {
    if i % 2 != 0 { i = i + 1; continue; }
    sum = sum + i;
    i = i + 1;
  }
  return sum;
}
fn main() -> Int {
  if sum_even(10) != 30 { return 1; }
  if sum_even(0) != 0 { return 2; }
  if sum_even(1) != 0 { return 3; }
  return 0;
}