// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0016

interface Measurable {
  fn area(&self) -> Float64 { return 0.0; }
  fn name(&self) -> Str;
}

type Square = { label: Str; side: Float64; }

fn Square.name(&self) -> Str { return label; }

fn Square.area(&self) -> Float64 { return side * side; }

fn main() -> Int {
  var s: Square = Square{ label: "sq", side: 3.0 };
  if s.name() == "sq" && s.area() == 9.0 { return 0; }
  return 1;
}
