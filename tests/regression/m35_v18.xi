// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V18: Vec[Int] as function parameter
fn sum_vec(v: &Vec[Int]) -> Int {
  var total = 0;
  var i = 0;
  while i < v.len() {
    total = total + v[i];
    i = i + 1;
  }
  return total;
}

fn count_vec(v: &Vec[Int]) -> Int {
  return v.len();
}

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(5);
  v.push(10);
  v.push(15);
  if count_vec(&v) != 3 { return 1; }
  if sum_vec(&v) != 30 { return 2; }
  v.push(20);
  if sum_vec(&v) != 50 { return 3; }
  if count_vec(&v) != 4 { return 4; }
  return 0;
}
