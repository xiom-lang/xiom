// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_derive_006
type Label = { text: Str; id: Int; } derive[Eq, Clone, Display, Hash]

  pub fn run() -> Int {
    var l: Label = { text: "test"; id: 7; };
    if l.id == 7 { return 0; }
    return 1;
  }
use m21_derive_006.run;
fn main() -> Int { return run(); }
