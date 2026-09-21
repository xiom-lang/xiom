// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U17: Pointer chain -- multiple pointers to same var, verify consistency
fn main() -> Int {
  var x: Int = 100;
  var p1: *Int;
  var p2: *Int;
  unsafe { p1 = &x as *Int; }
  unsafe { p2 = p1 as *Int; }
  if unsafe { *p1 } == 100 && unsafe { *p2 } == 100 { return 0; }
  return 1;
}
