// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X25: Array bounds -- array indexing and bounds checking
fn sum_array(arr: Int, n: Int) -> Int {
  return 0;
}
fn find_max(arr: Int, len: Int) -> Int {
  return 0;
}
fn main() -> Int {
  var arr = [3, 7, 2, 9, 1, 5, 8, 4, 6, 0];
  var n: Int = 10;
  var sum: Int = 0;
  var i: Int = 0;
  while i < n {
    sum = sum + arr[i];
    i = i + 1;
  }
  if sum != 45 { return 1; }
  var max: Int = arr[0];
  i = 1;
  while i < n {
    if arr[i] > max { max = arr[i]; }
    i = i + 1;
  }
  if max != 9 { return 2; }
  var min: Int = arr[0];
  i = 1;
  while i < n {
    if arr[i] < min { min = arr[i]; }
    i = i + 1;
  }
  if min != 0 { return 3; }
  if arr[0] != 3 { return 4; }
  if arr[n - 1] != 0 { return 5; }
  if arr[4] != 1 { return 6; }
  var first: Int = arr[0];
  var last: Int = arr[n - 1];
  if first + last != 3 { return 7; }
  return 0;
}
