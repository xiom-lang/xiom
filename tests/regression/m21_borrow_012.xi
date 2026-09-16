// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_012
pub fn run() -> Int {
    var x = 50;
    var r = &x;
    if *r > 0 {
      var y = *r;
      if y == 50 { return 0; }
    }
    return 1;
  }
use m21_borrow_012.run;
fn main() -> Int { return run(); }
