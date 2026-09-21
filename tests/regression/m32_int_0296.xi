// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Struct with Int8 field and cross-boundary field access
type Rec = { id: Int8; score: Int16; }
fn main() -> Int {
  var r: Rec = Rec{ id: -50 as Int8; score: -200 as Int16; };
  var sum: Int = r.id as Int + r.score as Int;
  if sum == -250 { return 0; }
  return 1;
}
