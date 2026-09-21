// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A09: Array in function param -- pass inline array elements explicitly
fn sum_two(a: Int, b: Int, c: Int) -> Int {
  return a + b + c;
}
fn main() -> Int {
  var arr = [30, 70, 200];
  if sum_two(arr[0], arr[1], arr[2]) == 300 { return 0; }
  return 1;
}
