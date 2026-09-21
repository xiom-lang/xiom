// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V28: Vec insert/remove/clear stress
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  // Insert at beginning -- growing list
  v.insert(0, 30);
  v.insert(0, 20);
  v.insert(0, 10);
  if v.len() != 3 { return 1; }
  if v[0] != 10 { return 2; }
  if v[1] != 20 { return 3; }
  if v[2] != 30 { return 4; }
  // Insert in middle
  v.insert(1, 15);
  if v[0] != 10 { return 5; }
  if v[1] != 15 { return 6; }
  if v[2] != 20 { return 7; }
  // Remove from middle
  var r = v.remove(1);
  if r != Some(15) { return 8; }
  if v.len() != 3 { return 9; }
  if v[0] != 10 { return 10; }
  if v[1] != 20 { return 11; }
  // Remove from beginning
  v.remove(0);
  if v[0] != 20 { return 12; }
  if v[1] != 30 { return 13; }
  // Remove from end
  v.remove(1);
  if v.len() != 1 { return 14; }
  if v[0] != 20 { return 15; }
  // Clear via pop
  while v.len() > 0 { v.pop(); }
  if v.len() != 0 { return 16; }
  // Re-fill after clear
  v.push(1);
  v.push(2);
  if v.len() != 2 { return 17; }
  return 0;
}
