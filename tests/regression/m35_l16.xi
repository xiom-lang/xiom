// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L16: Pointer comparison -- compare pointers for equality and null
fn main() -> Int {
  var v: Int = 42;
  var p: *Int;
  var q: *Int;
  unsafe { p = &v as *Int; }
  unsafe { q = p as *Int; }
  if unsafe { p != q } { return 1; }
  var np: *Int;
  unsafe { np = 0 as *Int; }
  if unsafe { np != (0 as *Int) } { return 2; }
  if unsafe { p != np } {
    var r: Int;
    unsafe { r = *p; }
    if r == 42 { return 0; }
  }
  return 3;
}
