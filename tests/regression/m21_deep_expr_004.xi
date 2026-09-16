// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_004
type A = { x: Int; }

  pub fn run() -> Int {
    var a: A = { x: 1; };
    a.x = a.x + a.x + a.x + a.x + a.x + a.x + a.x + a.x + a.x + a.x;
    if a.x == 10 { return 0; }
    return 1;
  }
use m21_deep_expr_004.run;
fn main() -> Int { return run(); }
