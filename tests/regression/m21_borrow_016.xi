// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_borrow_016
type Item = { a: Int; b: Int; }

  pub fn run() -> Int {
    var it: Item = { a: 10; b: 20; };
    var ra = &it.a;
    var rb = &it.b;
    if *ra == 10 && *rb == 20 { return 0; }
    return 1;
  }
use m21_borrow_016.run;
fn main() -> Int { return run(); }
