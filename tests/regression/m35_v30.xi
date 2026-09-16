// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V30: Vec[Int] all operations mega combined stress
use xiom.collections;

fn vec_sum_all(v: &Vec[Int]) -> Int {
  var total = 0;
  var i = 0;
  while i < v.len() {
    total = total + v[i];
    i = i + 1;
  }
  return total;
}

fn vec_max_val(v: &Vec[Int]) -> Int {
  if v.len() == 0 { return 0; }
  var m = v[0];
  var i = 1;
  while i < v.len() {
    if v[i] > m { m = v[i]; }
    i = i + 1;
  }
  return m;
}

fn main() -> Int {
  var v = Vec[Int].new();
  if v.len() != 0 { return 1; }

  v.push(5);
  v.push(15);
  v.push(25);
  v.push(35);
  if v.len() != 4 { return 2; }
  if v[0] != 5 { return 3; }
  if v[3] != 35 { return 4; }

  v.insert(2, 20);
  if v.len() != 5 { return 5; }
  if v[2] != 20 { return 6; }
  if v[3] != 25 { return 7; }

  // Replace via remove+insert
  v.remove(1);
  v.insert(1, 12);
  if v[1] != 12 { return 8; }

  var rem = v.remove(0);
  if rem != Some(5) { return 9; }
  if v.len() != 4 { return 10; }
  if v[0] != 12 { return 11; }

  var pop_val = v.pop();
  if pop_val != Some(35) { return 12; }
  if v.len() != 3 { return 13; }

  if vec_sum_all(&v) != 57 { return 14; }
  if vec_max_val(&v) != 25 { return 15; }

  // Clear
  while v.len() > 0 { v.pop(); }
  if v.len() != 0 { return 16; }

  v.push(100);
  v.push(200);
  v.push(300);
  var i = 0;
  while i < v.len() {
    if v[i] != (i + 1) * 100 { return 17 + i; }
    i = i + 1;
  }
  return 0;
}
