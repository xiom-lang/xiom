// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_011
type Data = { val: Int; flag: Bool; }

  pub fn run() -> Int {
    var d: Data = { val: 10; flag: false; };
    var r = &d.val;
    if *r == 10 { return 0; }
    return 1;
  }
use m21_borrow_011.run;
fn main() -> Int { return run(); }
