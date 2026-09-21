// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M04: max of 2/3/4 numbers
fn max2(a: Int, b: Int) -> Int {
  if a > b { return a; }
  return b;
}
fn max3(a: Int, b: Int, c: Int) -> Int {
  return max2(max2(a, b), c);
}
fn max4(a: Int, b: Int, c: Int, d: Int) -> Int {
  return max2(max3(a, b, c), d);
}
fn main() -> Int {
  if max2(3, 7) == 7 && max2(7, 3) == 7 && max2(5, 5) == 5 { } else { return 1; }
  if max3(3, 7, 1) == 7 && max3(10, 20, 30) == 30 { } else { return 2; }
  if max4(4, 2, 8, 1) == 8 && max4(5, 5, 5, 5) == 5 { } else { return 3; }
  return 0;
}
