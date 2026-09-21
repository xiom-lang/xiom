// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U12: Pointer comparison to null -- test null and non-null
fn main() -> Int {
  var x: Int = 5;
  var p: *Int;
  unsafe { p = &x as *Int; }
  if unsafe { p != (0 as *Int) } {
    var q: *Int;
    unsafe { q = 0 as *Int; }
    if unsafe { q == (0 as *Int) } { return 0; }
  }
  return 1;
}
