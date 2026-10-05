// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m197 (Pulse C-PULSE-04): a reference used BARE in value position must read
// the pointee, matching the write side (store-through). Pre-fix `p = p + 1`
// and `return p` yielded the stack address (silent wrong values).
fn bare_add(p: &mut Int) -> Int {
  p = p + 1;
  return p;
}

fn bare_read(p: &mut Int) -> Int {
  return p;
}

fn main() -> Int {
  var a: Int = 10;
  if bare_add(&mut a) != 11 { return 1; }
  if a != 11 { return 2; }

  var c: Int = 10;
  if bare_read(&mut c) != 10 { return 3; }

  // Reference passing still passes the reference itself.
  var d: Int = 10;
  let r = bare_add(&mut d);
  if r != 11 || d != 11 { return 4; }
  return 0;
}
