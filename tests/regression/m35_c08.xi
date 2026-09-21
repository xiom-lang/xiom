// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C08: while with break -- early loop termination
fn find_first_even(start: Int, limit: Int) -> Int {
  var i: Int = start;
  while i <= limit {
    if i % 2 == 0 { break; }
    i = i + 1;
  }
  return i;
}
fn main() -> Int {
  if find_first_even(1, 10) != 2 { return 1; }
  if find_first_even(3, 5) != 4 { return 2; }
  if find_first_even(0, 10) != 0 { return 3; }
  return 0;
}
