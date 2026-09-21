// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-C06: Pointer null requires -- must not pass null pointer
fn ptr_value(p: *mut Int) -> Bool
  requires: p != (0 as *mut Int)
{
  return true;
}
fn main() -> Int {
  var dp: *mut Int;
  unsafe { dp = 1 as *mut Int; }
  var ok: Bool = ptr_value(dp);
  if ok { return 0; }
  return 1;
}
