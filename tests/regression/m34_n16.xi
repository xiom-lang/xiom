// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N16: Derive comparison chain -- field-based Ord on struct
type Score = { points: Int; level: Int; } derive[Eq]
fn main() -> Int {
  var a = Score{ points: 10; level: 1; };
  var b = Score{ points: 20; level: 2; };
  var c = Score{ points: 20; level: 2; };
  var d = Score{ points: 5; level: 0; };
  if a.points < b.points && a.points > d.points && b.points == c.points { return 0; }
  return 1;
}
