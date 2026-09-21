// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V19: Vec[Int] as return type
fn make_range(start: Int, count: Int) -> Vec[Int] {
  var v = Vec[Int].new();
  var i = 0;
  while i < count {
    v.push(start + i);
    i = i + 1;
  }
  return v;
}

fn make_single(val: Int) -> Vec[Int] {
  var v = Vec[Int].new();
  v.push(val);
  return v;
}

fn main() -> Int {
  var r = make_range(10, 5);
  if r.len() != 5 { return 1; }
  if r[0] != 10 { return 2; }
  if r[1] != 11 { return 3; }
  if r[4] != 14 { return 4; }
  var s = make_single(99);
  if s.len() != 1 { return 5; }
  if s[0] != 99 { return 6; }
  return 0;
}
