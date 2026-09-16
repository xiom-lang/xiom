// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B05: Borrow through function param -- pass &mut, read after write, verify sequencing
fn double_it(x: &mut Int) { *x = *x * 2; }
fn add_ten(x: &mut Int) { *x = *x + 10; }
fn main() -> Int {
  var val = 7;
  double_it(&mut val);
  add_ten(&mut val);
  if val == 24 { return 0; }
  return 1;
}
