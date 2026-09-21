// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M29: While-break-continue runtime
fn main() -> Int {
  var i: Int = 0;
  var sum: Int = 0;
  while i < 20 {
    i = i + 1;
    if i % 3 == 0 { continue; }
    if i > 15 { break; }
    sum = sum + i;
  }
  // Sum of 1,2,4,5,7,8,10,11,13,14 = 75
  if sum == 75 { return 0; }
  return 1;
}
