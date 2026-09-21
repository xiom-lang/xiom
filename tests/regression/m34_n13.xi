// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N13: Struct with derive and invariant -- invariant condition coexists with derive
type Positive = { val: Int; invariant: val > 0; } derive[Eq, Clone]
fn main() -> Int {
  var a = Positive{ val: 42; };
  var b = Positive{ val: 42; };
  var c = Positive{ val: 7; };
  if a == b && a != c { return 0; }
  return 1;
}
