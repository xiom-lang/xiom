// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_015
pub fn run() -> Int {
    var x = 1;
    {
      var r = &mut x;
      *r = 10;
    }
    {
      var r2 = &x;
      if *r2 == 10 { return 0; }
    }
    return 1;
  }
use m21_borrow_015.run;
fn main() -> Int { return run(); }
