// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_018
pub fn run() -> Int {
    var v: Vec[Int] = [10, 20, 30];
    var r = &v[0];
    if *r == 10 { return 0; }
    return 1;
  }
use m21_borrow_018.run;
fn main() -> Int { return run(); }
