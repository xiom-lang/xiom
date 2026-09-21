// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_003
pub fn run() -> Int {
    var x = 5;
    {
      var r = &x;
      if *r != 5 { return 1; }
    }
    return 0;
  }
use m21_borrow_003.run;
fn main() -> Int { return run(); }
