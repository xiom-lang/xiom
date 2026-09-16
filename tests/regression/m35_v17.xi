// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V17: Vec[Int] as function return and parameter types
use xiom.collections;

fn vec_len(v: &Vec[Int]) -> Int {
  return v.len();
}

fn make_pair(a: Int, b: Int) -> Vec[Int] {
  var v = Vec[Int].new();
  v.push(a);
  v.push(b);
  return v;
}

fn make_triple(a: Int, b: Int, c: Int) -> Vec[Int] {
  var v = Vec[Int].new();
  v.push(a);
  v.push(b);
  v.push(c);
  return v;
}

fn main() -> Int {
  var v1 = make_pair(10, 20);
  if vec_len(&v1) != 2 { return 1; }
  if v1[0] != 10 { return 2; }
  if v1[1] != 20 { return 3; }
  var v2 = make_triple(100, 200, 300);
  if vec_len(&v2) != 3 { return 4; }
  if v2[0] != 100 { return 5; }
  if v2[1] != 200 { return 6; }
  if v2[2] != 300 { return 7; }
  return 0;
}
