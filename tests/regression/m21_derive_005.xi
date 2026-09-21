// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_derive_005
type Pair = { a: Int; b: Int; } derive[Eq, Clone, Ord]

  pub fn run() -> Int {
    var p1: Pair = { a: 1; b: 2; };
    var p2: Pair = { a: 1; b: 2; };
    if p1.a == p2.a && p1.b == p2.b { return 0; }
    return 1;
  }
use m21_derive_005.run;
fn main() -> Int { return run(); }
