// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V07: Vec[Int] remove
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  v.push(40);
  var r1 = v.remove(1);
  if r1 != Some(20) { return 1; }
  if v.len() != 3 { return 2; }
  if v[0] != 10 { return 3; }
  if v[1] != 30 { return 4; }
  if v[2] != 40 { return 5; }
  var r0 = v.remove(0);
  if r0 != Some(10) { return 6; }
  if v.len() != 2 { return 7; }
  if v[0] != 30 { return 8; }
  if v[1] != 40 { return 9; }
  var r_last = v.remove(1);
  if r_last != Some(40) { return 10; }
  if v.len() != 1 { return 11; }
  if v[0] != 30 { return 12; }
  var r_bad = v.remove(5);
  if r_bad != None { return 13; }
  return 0;
}
