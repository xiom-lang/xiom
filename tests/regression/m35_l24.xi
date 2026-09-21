// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L24: Pointer cast -- cast pointer to Int and back, verify value integrity
fn main() -> Int {
  var x: Int = 123;
  var p: *Int;
  var q: *Int;
  unsafe { p = &x as *Int; }
  unsafe { q = p; }
  var val: Int;
  unsafe { val = *q; }
  if val != 123 { return 1; }
  var px: *Int;
  var py: *Int;
  var y: Int = 456;
  unsafe { px = &x as *Int; }
  unsafe { py = &y as *Int; }
  if unsafe { *px } != 123 { return 2; }
  if unsafe { *py } != 456 { return 3; }
  if unsafe { px != py } { return 0; }
  return 4;
}
