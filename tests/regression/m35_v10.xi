// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V10: Vec[Float64] push, pop, index access
use xiom.collections;

fn main() -> Int {
  var v = Vec[Float64].new();
  v.push(1.5);
  v.push(2.5);
  v.push(3.5);
  if v.len() != 3 { return 1; }
  if v[0] != 1.5 { return 2; }
  if v[1] != 2.5 { return 3; }
  if v[2] != 3.5 { return 4; }
  // Replace via remove + insert
  v.remove(1);
  v.insert(1, 9.9);
  if v.get(1) != Some(9.9) { return 5; }
  var last = v.pop();
  if last != Some(3.5) { return 6; }
  if v[0] != 1.5 { return 7; }
  var sum = v[0] + v[1];
  if sum != 11.4 { return 8; }
  return 0;
}
