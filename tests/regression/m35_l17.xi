// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L17: Null pointer check -- verify null pointer detection and guard
fn is_null(p: *Int) -> Bool {
  if unsafe { p == (0 as *Int) } { return true; }
  return false;
}

fn deref_if_valid(p: *Int) -> Int {
  if is_null(p) { return -1; }
  var val: Int;
  unsafe { val = *p; }
  return val;
}

fn main() -> Int {
  var nullp: *Int;
  unsafe { nullp = 0 as *Int; }
  if is_null(nullp) != true { return 1; }
  if deref_if_valid(nullp) != -1 { return 2; }
  var x: Int = 99;
  var valid: *Int;
  unsafe { valid = &x as *Int; }
  if is_null(valid) != false { return 3; }
  if deref_if_valid(valid) != 99 { return 4; }
  return 0;
}
