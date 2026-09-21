// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E07: Deeply nested if/elif -- 20 levels (via elif cascade)
fn nest20(x: Int) -> Int {
  if x == 1 { return 10; }
  elif x == 2 { return 20; }
  elif x == 3 { return 30; }
  elif x == 4 { return 40; }
  elif x == 5 { return 50; }
  elif x == 6 { return 60; }
  elif x == 7 { return 70; }
  elif x == 8 { return 80; }
  elif x == 9 { return 90; }
  elif x == 10 { return 100; }
  elif x == 11 { return 110; }
  elif x == 12 { return 120; }
  elif x == 13 { return 130; }
  elif x == 14 { return 140; }
  elif x == 15 { return 150; }
  elif x == 16 { return 160; }
  elif x == 17 { return 170; }
  elif x == 18 { return 180; }
  elif x == 19 { return 190; }
  elif x == 20 { return 200; }
  else { return 0; }
}
fn main() -> Int {
  if nest20(1) != 10 { return 1; }
  if nest20(10) != 100 { return 2; }
  if nest20(20) != 200 { return 3; }
  if nest20(15) != 150 { return 4; }
  if nest20(0) != 0 { return 5; }
  return 0;
}