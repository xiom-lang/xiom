// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N01: Struct with derive[Eq] -- basic struct equality
type Point = { x: Int; y: Int; } derive[Eq]
fn main() -> Int {
  var a = Point{ x: 1; y: 2; };
  var b = Point{ x: 1; y: 2; };
  var c = Point{ x: 3; y: 4; };
  if a == b && a != c { return 0; }
  return 1;
}
