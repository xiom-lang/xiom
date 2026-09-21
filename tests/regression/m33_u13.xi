// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U13: Pointer return -- a function returns a pointer it was given.
// (Original form returned `&local` -- dangling by design; rewritten to be
// well-defined: pointer in, pointer out.)
fn get_addr(p: *Int) -> *Int {
  var q: *Int = p;
  unsafe { return q; }
}
fn main() -> Int {
  var x: Int = 42;
  var p: *Int = &x;
  var q: *Int = get_addr(p);
  if unsafe { *q } == 42 { return 0; }
  return 1;
}
