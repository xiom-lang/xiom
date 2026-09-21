// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V09: Vec[Int] resize -- push many elements past initial capacity
fn main() -> Int {
  var v = Vec[Int].new();
  var i = 0;
  while i < 50 {
    v.push(i);
    i = i + 1;
  }
  if v.len() != 50 { return 1; }
  if v[0] != 0 { return 2; }
  if v[25] != 25 { return 3; }
  if v[49] != 49 { return 4; }
  // Pop last 10
  i = 0;
  while i < 10 {
    v.pop();
    i = i + 1;
  }
  if v.len() != 40 { return 5; }
  if v[39] != 39 { return 6; }
  // Pop remaining via loop
  while v.len() > 0 {
    v.pop();
  }
  if v.len() != 0 { return 7; }
  return 0;
}
