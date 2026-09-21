// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U18: Pointer to multiple Int vars -- simulate array with separate vars
fn main() -> Int {
  var e0: Int = 1;
  var e1: Int = 2;
  var e2: Int = 3;
  var p0: *Int;
  var p1: *Int;
  var p2: *Int;
  unsafe { p0 = &e0 as *Int; }
  unsafe { p1 = &e1 as *Int; }
  unsafe { p2 = &e2 as *Int; }
  if unsafe { *p0 } == 1 && unsafe { *p1 } == 2 && unsafe { *p2 } == 3 { return 0; }
  return 1;
}
