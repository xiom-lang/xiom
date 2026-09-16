// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C19: loop with counter -- counting iterations with various termination conditions
fn count_divisible(limit: Int, d: Int) -> Int {
  var i: Int = 1;
  var count: Int = 0;
  while i <= limit {
    if i % d == 0 { count = count + 1; }
    i = i + 1;
  }
  return count;
}
fn main() -> Int {
  if count_divisible(10, 2) != 5 { return 1; }
  if count_divisible(10, 3) != 3 { return 2; }
  if count_divisible(100, 10) != 10 { return 3; }
  return 0;
}
