// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_013
pub fn run() -> Int {
    var x = 5;
    var r = &x;
    var i = 0;
    while i < *r {
      i = i + 1;
    }
    if i == 5 { return 0; }
    return 1;
  }
use m21_borrow_013.run;
fn main() -> Int { return run(); }
