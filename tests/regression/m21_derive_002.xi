// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_derive_002
type Person = {
    name: Str;
    age: Int;
  } derive[Eq, Clone, Display]

  pub fn run() -> Int {
    var p: Person = { name: "Alice"; age: 30; };
    if p.age == 30 { return 0; }
    return 1;
  }
use m21_derive_002.run;
fn main() -> Int { return run(); }
