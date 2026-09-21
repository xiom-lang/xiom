// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C16: loop with multiple accumulators -- tracking several values simultaneously
fn multi_accum(n: Int) -> Int {
  var sum: Int = 0;
  var prod: Int = 1;
  var count: Int = 0;
  var i: Int = 1;
  while i <= n {
    sum = sum + i;
    prod = prod * i;
    count = count + 1;
    i = i + 1;
  }
  return sum + prod + count;
}
fn main() -> Int {
  if multi_accum(3) != 15 { return 1; }
  if multi_accum(4) != 38 { return 2; }
  return 0;
}
