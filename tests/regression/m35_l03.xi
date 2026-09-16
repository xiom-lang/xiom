// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L03: Struct with mixed fields -- Int, Float64, Bool, Char, Str
type Mixed = { id: Int; score: Float64; flag: Bool; ch: Char; name: Str; }

fn main() -> Int {
  var m = Mixed{ id: 42; score: 99.9; flag: true; ch: 'X'; name: "test"; };
  if m.id != 42 { return 1; }
  if m.score != 99.9 { return 2; }
  if m.flag != true { return 3; }
  if m.ch != 'X' { return 4; }
  var sum: Int = m.id + 1;
  if sum != 43 { return 5; }
  return 0;
}
