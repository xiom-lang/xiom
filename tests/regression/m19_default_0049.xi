// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0049

interface Colored {
  fn to_str(&self) -> Str { return "color"; }
  fn code(&self) -> Int;
}

enum Color {
  Red,
  Green,
  Blue
}

fn Color.code(&self) -> Int {
  match self {
    Red => 1,
    Green => 2,
    Blue => 3
  }
}

fn main() -> Int {
  var r: Color = Color.Red;
  var g: Color = Color.Green;
  var b: Color = Color.Blue;
  if r.code() != 1 { return 1; }
  if r.to_str() != "color" { return 2; }
  if g.code() != 2 { return 3; }
  if g.to_str() != "color" { return 4; }
  if b.code() != 3 { return 5; }
  if b.to_str() != "color" { return 6; }
  return 0;
}
