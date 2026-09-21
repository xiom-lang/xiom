// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M06: factorial (iterative)
fn fact_iter(n: Int) -> Int {
  if n < 0 { return 0; }
  var result: Int = 1;
  var i: Int = 1;
  while i <= n {
    result = result * i;
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if fact_iter(0) == 1 && fact_iter(1) == 1 && fact_iter(5) == 120 && fact_iter(6) == 720 { return 0; }
  return 1;
}
