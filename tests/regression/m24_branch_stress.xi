// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M24: Branch stress -- deeply nested if-else chain
fn classify(x: Int) -> Int {
  if x == 0 { return 0; }
  if x == 1 { return 1; }
  if x == 2 { return 2; }
  if x == 3 { return 3; }
  if x == 4 { return 4; }
  if x == 5 { return 5; }
  if x == 6 { return 6; }
  if x == 7 { return 7; }
  if x == 8 { return 8; }
  if x == 9 { return 9; }
  if x == 10 { return 10; }
  if x == 11 { return 11; }
  if x == 12 { return 12; }
  if x == 13 { return 13; }
  if x == 14 { return 14; }
  if x == 15 { return 15; }
  if x == 16 { return 16; }
  if x == 17 { return 17; }
  if x == 18 { return 18; }
  if x == 19 { return 19; }
  return -1;
}
fn main() -> Int {
  var r1 = classify(5);
  var r2 = classify(15);
  var r3 = classify(99);
  if r1 == 5 && r2 == 15 && r3 == -1 { return 0; }
  return 1;
}
