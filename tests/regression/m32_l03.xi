// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-L03: While with continue -- skip even numbers, sum odds 1..19
fn main() -> Int {
  var i: Int = 0;
  var sum: Int = 0;
  while i < 20 {
    i += 1;
    if i % 2 == 0 { continue; }
    sum += i;
  }
  // odd 1..19: 1+3+5+7+9+11+13+15+17+19 = 100
  if sum == 100 { return 0; }
  return 1;
}
