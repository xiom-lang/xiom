// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M16: geometric series sum: a * (r^n - 1) / (r - 1) for r != 1
fn geom_sum(a: Int, r: Int, n: Int) -> Int {
  if n <= 0 { return 0; }
  if r == 1 { return a * n; }
  var pow: Int = 1;
  var i: Int = 0;
  while i < n {
    pow = pow * r;
    i = i + 1;
  }
  return a * (pow - 1) / (r - 1);
}
fn main() -> Int {
  if geom_sum(1, 2, 3) == 7 && geom_sum(1, 3, 4) == 40 && geom_sum(2, 1, 5) == 10 && geom_sum(3, 2, 1) == 3 { return 0; }
  return 1;
}
