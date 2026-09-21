// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A13: Very large array -- sum of 100 elements 0..99
fn main() -> Int {
  var i: Int = 0;
  var sum: Int = 0;
  while i < 100 {
    sum += i;
    i += 1;
  }
  if sum == 4950 && i == 100 { return 0; }
  return 1;
}
