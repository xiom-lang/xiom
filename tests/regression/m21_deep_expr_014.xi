// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_014
enum Val { A, B, C, D, E, F, G, H, I, J }

  pub fn run() -> Int {
    var x = Val.F;
    if x == Val.A { return 1; }
    elif x == Val.B { return 1; }
    elif x == Val.C { return 1; }
    elif x == Val.D { return 1; }
    elif x == Val.E { return 1; }
    elif x == Val.F { return 0; }
    elif x == Val.G { return 1; }
    elif x == Val.H { return 1; }
    elif x == Val.I { return 1; }
    elif x == Val.J { return 1; }
    else { return 1; }
  }
use m21_deep_expr_014.run;
fn main() -> Int { return run(); }
