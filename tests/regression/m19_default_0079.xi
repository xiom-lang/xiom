// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0079

interface Override {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return a() * 2; }
}

type Num = { x: Int; }

fn Num.b(&self) -> Int { return x; }

fn Num.a(&self) -> Int { return 1; }

fn main() -> Int {
  var n: Num = Num{ x: 100 };
  if n.a() == 1 && n.b() == 100 { return 0; }
  return 1;
}
