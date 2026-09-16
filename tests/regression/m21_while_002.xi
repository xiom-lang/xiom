// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_002
pub fn run() -> Int {
    var i = 0;
    while true {
      i = i + 1;
      if i >= 5 { break; }
    }
    if i == 5 { return 0; }
    return 1;
  }
use m21_while_002.run;
fn main() -> Int { return run(); }
