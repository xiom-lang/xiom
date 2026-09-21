// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E02: Single-field struct
type Single = { x: Int; }
fn main() -> Int {
  var s = Single{ x: 42 };
  if s.x != 42 { return 1; }
  return 0;
}
