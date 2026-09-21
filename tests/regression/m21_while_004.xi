// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_004
pub fn run() -> Int {
    var a = 1;
    var b = 1;
    var i = 0;
    while a + b < 100 {
      var tmp = a + b;
      a = b;
      b = tmp;
      i = i + 1;
    }
    if i == 9 { return 0; }
    return 1;
  }
use m21_while_004.run;
fn main() -> Int { return run(); }
