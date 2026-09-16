// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C10: while with nested break -- inner loop break only
fn nested_break_test() -> Int {
  var outer: Int = 0;
  var inner: Int = 0;
  var total: Int = 0;
  while outer < 3 {
    inner = 0;
    while inner < 10 {
      total = total + 1;
      inner = inner + 1;
      if inner >= 5 { break; }
    }
    outer = outer + 1;
  }
  return total;
}
fn main() -> Int {
  if nested_break_test() != 15 { return 1; }
  return 0;
}
