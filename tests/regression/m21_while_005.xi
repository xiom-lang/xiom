// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_005
pub fn run() -> Int {
    var outer = 0;
    var inner = 0;
    while outer < 5 {
      outer = outer + 1;
      inner = 0;
      while inner < 3 {
        inner = inner + 1;
      }
    }
    if outer == 5 && inner == 3 { return 0; }
    return 1;
  }
use m21_while_005.run;
fn main() -> Int { return run(); }
