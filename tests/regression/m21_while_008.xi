// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_008
pub fn run() -> Int {
    var i = 0;
    while i < 100 {
      i = i + 1;
      if i == 42 { break; }
    }
    if i == 42 { return 0; }
    return 1;
  }
use m21_while_008.run;
fn main() -> Int { return run(); }
