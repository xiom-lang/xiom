// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_001
pub fn run() -> Int {
    var i = 0;
    while i < 10 {
      i = i + 1;
    }
    if i == 10 { return 0; }
    return 1;
  }
use m21_while_001.run;
fn main() -> Int { return run(); }
