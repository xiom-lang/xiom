// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B20: Borrow chain -- read + write borrows sequenced across separate fn calls
fn add_pair(a: Int, b: Int) -> Int { return a + b; }
fn scale_both(x: &mut Int, y: &mut Int) { *x = *x * 2; *y = *y + 1; }
fn main() -> Int {
  var a = 10;
  var b = 3;
  var sum = add_pair(a, b);
  scale_both(&mut a, &mut b);
  if sum == 13 && a == 20 && b == 4 { return 0; }
  return 1;
}
