// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: Pattern matching with conditions
fn classify(n: Int) -> Int {
  if n > 0 {
    return 1;
  }
  if n < 0 {
    return -1;
  }
  return 0;
}
fn main() -> Int {
  var r1: Int = classify(10);
  var r2: Int = classify(-5);
  var r3: Int = classify(0);
  if r1 == 1 && r2 == -1 && r3 == 0 {
    return 0;
  }
  return 1;
}
