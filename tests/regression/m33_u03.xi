// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U03: Pointer creation -- declare pointer var and assign via cast
fn main() -> Int {
  var a: Int = 10;
  var p: *Int;
  unsafe { p = &a as *Int; }
  if unsafe { *p } == 10 { return 0; }
  return 1;
}
