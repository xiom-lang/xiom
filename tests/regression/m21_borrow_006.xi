// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_006
pub fn run() -> Int {
    var x = 10;
    {
      var r1 = &x;
      if *r1 != 10 { return 1; }
    }
    {
      var r2 = &x;
      if *r2 != 10 { return 1; }
    }
    return 0;
  }
use m21_borrow_006.run;
fn main() -> Int { return run(); }
