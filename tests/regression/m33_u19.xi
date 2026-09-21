// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U19: Pointer with contract -- contract-guarded pointer access via temp
fn safe_deref(p: *Int) -> Int
  requires: p != (0 as *Int)
{
  var r: Int;
  unsafe { r = *p; }
  return r;
}
fn main() -> Int {
  var v: Int = 55;
  var p: *Int;
  unsafe { p = &v as *Int; }
  var r: Int = safe_deref(p);
  if r == 55 { return 0; }
  return 1;
}
