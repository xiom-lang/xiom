// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_004
fn read_ref(r: &Int) -> Int {
    return *r;
  }

  pub fn run() -> Int {
    var x = 42;
    var v = read_ref(&x);
    if v == 42 { return 0; }
    return 1;
  }
use m21_borrow_004.run;
fn main() -> Int { return run(); }
