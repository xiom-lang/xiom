// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_009
pub fn run() -> Int {
    var i = 0;
    var evens = 0;
    while i < 20 {
      i = i + 1;
      if i % 2 != 0 { continue; }
      evens = evens + 1;
    }
    if evens == 10 { return 0; }
    return 1;
  }
use m21_while_009.run;
fn main() -> Int { return run(); }
