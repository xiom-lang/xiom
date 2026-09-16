// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M19: Pascal triangle row n (returns C(n,0..n))
fn binom(n: Int, k: Int) -> Int {
  if k < 0 || k > n { return 0; }
  if k == 0 || k == n { return 1; }
  var kk = k;
  if kk > n - kk { kk = n - kk; }
  var result: Int = 1;
  var i: Int = 0;
  while i < kk {
    result = result * (n - i);
    result = result / (i + 1);
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  var r0 = binom(4, 0);
  var r1 = binom(4, 1);
  var r2 = binom(4, 2);
  var r3 = binom(4, 3);
  var r4 = binom(4, 4);
  if r0 == 1 && r1 == 4 && r2 == 6 && r3 == 4 && r4 == 1 { return 0; }
  return 1;
}
