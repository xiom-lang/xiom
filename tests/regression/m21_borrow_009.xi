// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_009
fn increment(r: &mut Int) {
    *r = *r + 1;
  }

  pub fn run() -> Int {
    var x = 10;
    increment(&mut x);
    if x == 11 { return 0; }
    return 1;
  }
use m21_borrow_009.run;
fn main() -> Int { return run(); }
