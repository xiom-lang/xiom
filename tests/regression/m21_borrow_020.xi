// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_020
pub fn run() -> Int {
    var x = 77;
    var cond = true;
    if cond {
      var r = &x;
      if *r == 77 { }
    } else {
      var r = &x;
      if *r == 77 { }
    }
    return 0;
  }
use m21_borrow_020.run;
fn main() -> Int { return run(); }
