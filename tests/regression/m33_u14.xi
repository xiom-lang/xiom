// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U14: Pointer as param -- pass pointer to function, deref via temp
fn read_ptr(p: *Int) -> Int {
  var r: Int;
  unsafe { r = *p; }
  return r;
}
fn main() -> Int {
  var val: Int = 77;
  var p: *Int;
  unsafe { p = &val as *Int; }
  var r: Int = read_ptr(p);
  if r == 77 { return 0; }
  return 1;
}
