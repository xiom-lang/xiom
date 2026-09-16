// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C13: do-while pattern -- execute body at least once before checking condition
fn do_while_pattern(val: Int) -> Int {
  var x: Int = val;
  var count: Int = 0;
  var first: Bool = true;
  while first || x > 0 {
    first = false;
    count = count + x;
    x = x - 1;
  }
  return count;
}
fn main() -> Int {
  if do_while_pattern(5) != 15 { return 1; }
  if do_while_pattern(0) != 0 { return 2; }
  if do_while_pattern(3) != 6 { return 3; }
  return 0;
}
