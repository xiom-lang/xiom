// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V27: Vec[Int] combined: push/pop/index/get/remove stress
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  // Push 50 elements
  var i = 0;
  while i < 50 {
    v.push(i * 10);
    i = i + 1;
  }
  if v.len() != 50 { return 1; }
  if v[0] != 0 { return 2; }
  if v[49] != 490 { return 3; }
  // Get test
  if v[25] != 250 { return 4; }
  if v.get(49) != Some(490) { return 5; }
  if v.get(50) != None { return 6; }
  // Replace via remove+insert
  v.remove(10);
  v.insert(10, 999);
  if v[10] != 999 { return 7; }
  // Pop 10 from end
  i = 0;
  while i < 10 {
    v.pop();
    i = i + 1;
  }
  if v.len() != 40 { return 8; }
  if v[39] != 390 { return 9; }
  // Pop all remaining
  while v.len() > 0 {
    v.pop();
  }
  if v.len() != 0 { return 10; }
  return 0;
}
