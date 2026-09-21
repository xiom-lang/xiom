// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B01: Read borrow on local -- & ref to struct, read field without moving
type Wrapper = { val: Int; }
fn read_val(w: &Wrapper) -> Int { return w.val; }
fn main() -> Int {
  var a = Wrapper{ val: 42; };
  var b = read_val(&a);
  if a.val == 42 && b == 42 { return 0; }
  return 1;
}
