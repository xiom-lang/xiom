// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V05: Vec[Int] clear via pop-all pattern
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  if v.len() != 3 { return 1; }
  // Clear by popping all
  while v.len() > 0 {
    v.pop();
  }
  if v.len() != 0 { return 2; }
  var p = v.pop();
  if p != None { return 3; }
  v.push(100);
  if v.len() != 1 { return 4; }
  if v[0] != 100 { return 5; }
  while v.len() > 0 {
    v.pop();
  }
  v.push(7);
  v.push(8);
  v.push(9);
  while v.len() > 0 {
    v.pop();
  }
  v.push(1);
  if v.len() != 1 { return 6; }
  if v[0] != 1 { return 7; }
  return 0;
}
