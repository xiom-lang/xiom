// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_019
type Pair = { first: Int; second: Int; }

  fn get_first(p: &Pair) -> &Int {
    return &p.first;
  }

  pub fn run() -> Int {
    var p: Pair = { first: 5; second: 10; };
    var r = get_first(&p);
    if *r == 5 { return 0; }
    return 1;
  }
use m21_borrow_019.run;
fn main() -> Int { return run(); }
