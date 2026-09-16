// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_derive_003
type Card = {
    suit: Int;
    rank: Int;
  } derive[Eq, Clone, Hash, Ord]

  pub fn run() -> Int {
    var c: Card = { suit: 1; rank: 13; };
    if c.rank == 13 { return 0; }
    return 1;
  }
use m21_derive_003.run;
fn main() -> Int { return run(); }
