// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V21: Vec reverse pattern -- manual in-place reverse
fn reverse_vec(v: &Vec[Int]) -> Vec[Int] {
  var result = Vec[Int].new();
  var i = v.len() - 1;
  while i >= 0 {
    result.push(v[i]);
    i = i - 1;
  }
  return result;
}

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  v.push(4);
  v.push(5);
  var rev = reverse_vec(&v);
  if rev.len() != 5 { return 1; }
  if rev[0] != 5 { return 2; }
  if rev[1] != 4 { return 3; }
  if rev[2] != 3 { return 4; }
  if rev[3] != 2 { return 5; }
  if rev[4] != 1 { return 6; }
  // Original unchanged
  if v[0] != 1 { return 7; }
  if v[4] != 5 { return 8; }
  // Double reverse = identity
  var rev2 = reverse_vec(&rev);
  if rev2[0] != 1 { return 9; }
  if rev2[4] != 5 { return 10; }
  return 0;
}
