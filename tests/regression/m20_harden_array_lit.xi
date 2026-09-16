// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn sum_arr(arr: Vec[Int], n: Int) -> Int {
  var i = 0;
  var sum = 0;
  while i < n {
    sum = sum + arr[i];
    i = i + 1;
  }
  return sum;
}
fn main() -> Int {
  var arr: Vec[Int] = Vec[Int].new();
  arr.push(1); arr.push(2); arr.push(3); arr.push(4); arr.push(5);
  if sum_arr(arr, 5) != 15 { return 1; }
  return 0;
}