// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_011
enum Outer {
    A,
    B(inner: Option[Int]),
    C(x: Int, y: Int, z: Int),
  }

  pub fn run() -> Int {
    var o = Outer.C(1, 2, 3);
    match o {
      Outer.A => return 1,
      Outer.B(_) => return 1,
      Outer.C(x, y, z) => if x + y + z == 6 { return 0; },
    }
  }
use m21_match_edge_011.run;
fn main() -> Int { return run(); }
