// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z10: Compound assignment in while loop -- accumulate sum
fn main() -> Int {
  var i: Int = 0;
  var sum: Int = 0;
  while i < 10 {
    sum += i;
    i += 1;
  }
  if sum == 45 { return 0; }
  return 1;
}
