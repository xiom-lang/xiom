// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_else_if
// BUG 23 #4 regression: the two-word `else if` chain must parse
// (previously: error[P001] expected '{', found if).

fn main() -> Int {
  var x = 3;
  if x == 1 {
    return 1;
  } else if x == 2 {
    return 2;
  } else if x == 3 {
    return 0;
  } else {
    return 4;
  }
}
