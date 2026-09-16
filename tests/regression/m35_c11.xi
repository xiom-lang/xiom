// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C11: while with return inside -- function return from within loop body
fn find_divisor(n: Int, divisor: Int) -> Int {
  var i: Int = 1;
  while i <= n {
    if i % divisor == 0 && n % i == 0 { return i; }
    i = i + 1;
  }
  return -1;
}
fn main() -> Int {
  if find_divisor(12, 2) != 2 { return 1; }
  if find_divisor(15, 3) != 3 { return 2; }
  if find_divisor(7, 2) != -1 { return 3; }
  return 0;
}
