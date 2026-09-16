// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_009
enum Color { Red, Green, Blue }

  pub fn run() -> Int {
    var c = Color.Green;
    if c == Color.Red { return 1; }
    elif c == Color.Green { return 0; }
    elif c == Color.Blue { return 2; }
    else { return 3; }
  }
use m21_if_chain_009.run;
fn main() -> Int { return run(); }
