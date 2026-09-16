// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V11: Vec[Bool] push, pop, index access
use xiom.collections;

fn main() -> Int {
  var v = Vec[Bool].new();
  v.push(true);
  v.push(false);
  v.push(true);
  if v.len() != 3 { return 1; }
  if v[0] != true { return 2; }
  if v[1] != false { return 3; }
  if v[2] != true { return 4; }
  var p = v.pop();
  if p != Some(true) { return 5; }
  if v.len() != 2 { return 6; }
  if v[0] != true { return 7; }
  if v[1] != false { return 8; }
  // Replace second element
  v.remove(1);
  v.insert(1, true);
  if v[0] != true { return 9; }
  if v[1] != true { return 10; }
  // Clear
  while v.len() > 0 { v.pop(); }
  if v.len() != 0 { return 11; }
  return 0;
}
