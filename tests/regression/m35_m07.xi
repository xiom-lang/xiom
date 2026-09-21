// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M07: factorial (recursive)
fn fact_rec(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * fact_rec(n - 1);
}
fn main() -> Int {
  if fact_rec(0) == 1 && fact_rec(1) == 1 && fact_rec(5) == 120 && fact_rec(7) == 5040 { return 0; }
  return 1;
}
