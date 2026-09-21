// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-L09: While with complex condition -- loop while both counters below limit
fn main() -> Int {
  var a: Int = 0;
  var b: Int = 0;
  var count: Int = 0;
  var keep_going: Int = 1;
  while keep_going == 1 {
    a += 10;
    b += 7;
    count += 1;
    if a >= 50 || b >= 30 { keep_going = 0; }
  }
  // iter1: a=10 b=7  -> continue
  // iter2: a=20 b=14 -> continue
  // iter3: a=30 b=21 -> continue
  // iter4: a=40 b=28 -> continue
  // iter5: a=50 b=35 -> a >= 50 -> keep_going=0, count=5
  if count == 5 { return 0; }
  return 1;
}
