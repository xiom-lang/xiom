// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X24: Pointer safety -- pointer operations with unsafe blocks
fn read_ptr(p: *Int) -> Int {
  var r: Int;
  unsafe { r = *p; }
  return r;
}
fn is_null_ptr(p: *Int) -> Bool {
  if unsafe { p == (0 as *Int) } { return true; }
  return false;
}
fn main() -> Int {
  var x: Int = 42;
  var p: *Int;
  unsafe { p = &x as *Int; }
  if read_ptr(p) != 42 { return 1; }
  if is_null_ptr(p) { return 2; }
  var null_ptr: *Int;
  unsafe { null_ptr = 0 as *Int; }
  if !is_null_ptr(null_ptr) { return 3; }
  var y: Int = 77;
  var q: *Int;
  unsafe { q = &y as *Int; }
  var r: Int = read_ptr(q);
  if r != 77 { return 4; }
  var p2: *Int;
  unsafe { p2 = &x as *Int; }
  if unsafe { p != p2 } { return 5; }
  if unsafe { p == (0 as *Int) } { return 6; }
  return 0;
}
