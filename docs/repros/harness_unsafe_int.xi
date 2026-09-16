// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use unsafe_int_selfcontained;

fn main() -> Int {
  // abs(-7) = 7 -- must read EXACTLY 7 through the trampoline.
  var r1 = abs_rc(-7);
  if r1 != 7 { return 1; }
  // abs(3) = 3.
  var r2 = abs_rc(3);
  if r2 != 3 { return 2; }
  // strlen("hello") = 5.
  var r3 = strlen_rc("hello");
  if r3 != 5 { return 3; }
  return 0;
}

