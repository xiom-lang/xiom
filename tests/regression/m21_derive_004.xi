// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_derive_004
enum Color { Red, Green, Blue } derive[Eq, Clone, Hash]

  pub fn run() -> Int {
    var c = Color.Green;
    if c == Color.Green { return 0; }
    return 1;
  }
use m21_derive_004.run;
fn main() -> Int { return run(); }
