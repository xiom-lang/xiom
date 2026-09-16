// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A17: Palindrome number -- check if number reads the same forwards/backwards
fn rev(n: Int) -> Int {
  var x: Int = n;
  var r: Int = 0;
  while x > 0 {
    r = r * 10 + x % 10;
    x = x / 10;
  }
  return r;
}
fn is_pal(n: Int) -> Int {
  if n == rev(n) { return 1; }
  return 0;
}
fn main() -> Int {
  if is_pal(12321) != 1 { return 1; }
  if is_pal(12345) != 0 { return 2; }
  if is_pal(0) != 1 { return 3; }
  if is_pal(11) != 1 { return 4; }
  if is_pal(1234321) != 1 { return 5; }
  if is_pal(10) != 0 { return 6; }
  return 0;
}
