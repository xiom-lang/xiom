// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M20: Catalan number C_n = C(2n,n) / (n+1) = binom(2n,n) - binom(2n,n+1)
fn catalan(n: Int) -> Int {
  if n <= 0 { return 1; }
  var result: Int = 1;
  var i: Int = 0;
  while i < n {
    result = result * (4 * i + 2) / (i + 2);
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if catalan(0) == 1 && catalan(1) == 1 && catalan(2) == 2 && catalan(3) == 5 && catalan(4) == 14 && catalan(5) == 42 { return 0; }
  return 1;
}
