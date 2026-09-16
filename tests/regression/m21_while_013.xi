// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_013
pub fn run() -> Int {
    var i = 0;
    var result = 0;
    while i < 10 {
      if i % 2 == 0 {
        result = result + i;
      } else {
        result = result + 1;
      }
      i = i + 1;
    }
    if result == 25 { return 0; }
    return 1;
  }
use m21_while_013.run;
fn main() -> Int { return run(); }
