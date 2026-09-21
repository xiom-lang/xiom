// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_002
pub fn run() -> Int {
    var x = 100;
    var r1 = &x;
    var r2 = &x;
    if *r1 == 100 && *r2 == 100 { return 0; }
    return 1;
  }
use m21_borrow_002.run;
fn main() -> Int { return run(); }
