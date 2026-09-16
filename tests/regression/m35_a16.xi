// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A16: Reverse number -- reverse decimal digits via while loop
fn rev(n: Int) -> Int {
  var x: Int = n;
  var r: Int = 0;
  while x > 0 {
    r = r * 10 + x % 10;
    x = x / 10;
  }
  return r;
}
fn main() -> Int {
  if rev(12345) != 54321 { return 1; }
  if rev(0) != 0 { return 2; }
  if rev(1001) != 1001 { return 3; }
  if rev(900) != 9 { return 4; }
  if rev(987654321) != 123456789 { return 5; }
  return 0;
}
