// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B02: Write borrow on local -- &mut to mutate through reference
fn inc_val(x: &mut Int) { *x = *x + 1; }
fn main() -> Int {
  var a = 10;
  inc_val(&mut a);
  if a == 11 { return 0; }
  return 1;
}
