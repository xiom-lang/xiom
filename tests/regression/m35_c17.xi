// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C17: loop with early exit -- return immediately when condition met
fn find_index(needle: Int, max: Int) -> Int {
  var i: Int = 0;
  while i < max {
    if i == needle { return i; }
    i = i + 1;
  }
  return -1;
}
fn main() -> Int {
  if find_index(5, 10) != 5 { return 1; }
  if find_index(0, 5) != 0 { return 2; }
  if find_index(42, 10) != -1 { return 3; }
  return 0;
}
