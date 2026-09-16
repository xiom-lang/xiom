// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M03: min of 2/3/4 numbers
fn min2(a: Int, b: Int) -> Int {
  if a < b { return a; }
  return b;
}
fn min3(a: Int, b: Int, c: Int) -> Int {
  return min2(min2(a, b), c);
}
fn min4(a: Int, b: Int, c: Int, d: Int) -> Int {
  return min2(min3(a, b, c), d);
}
fn main() -> Int {
  if min2(3, 7) == 3 && min2(7, 3) == 3 && min2(5, 5) == 5 { } else { return 1; }
  if min3(3, 7, 1) == 1 && min3(10, 20, 30) == 10 { } else { return 2; }
  if min4(4, 2, 8, 1) == 1 && min4(5, 5, 5, 5) == 5 { } else { return 3; }
  return 0;
}
