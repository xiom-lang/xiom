// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_010
fn get_ref(x: &Int) -> &Int {
    return x;
  }

  pub fn run() -> Int {
    var x = 42;
    var r = get_ref(&x);
    if *r == 42 { return 0; }
    return 1;
  }
use m21_borrow_010.run;
fn main() -> Int { return run(); }
