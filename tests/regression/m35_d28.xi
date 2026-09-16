// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D28: Segment tree -- range sum verification
fn main() -> Int {
  var t0: Int = 36;
  var t1: Int = 9;
  var t2: Int = 27;
  var t3: Int = 4;
  var t4: Int = 5;
  var t5: Int = 16;
  var t6: Int = 11;
  if t0 != 36 { return 1; }
  if t1 != 9 { return 2; }
  if t2 != 27 { return 3; }
  t1 = 14;
  t0 = 41;
  if t1 != 14 { return 4; }
  if t0 != 41 { return 5; }
  return 0;
}
