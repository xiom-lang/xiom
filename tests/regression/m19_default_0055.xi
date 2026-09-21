// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0055

interface Summable {
  fn sum(&self) -> Int { return a() + b(); }
  fn a(&self) -> Int;
  fn b(&self) -> Int;
}

type Pair = { x: Int; y: Int; }

fn Pair.sum(self) -> Int { return self.a() + self.b(); }


fn Pair.a(&self) -> Int { return x; }

fn Pair.b(&self) -> Int { return y; }

fn main() -> Int {
  var p: Pair = Pair{ x: 10, y: 20 };
  if p.a() == 10 && p.b() == 20 && p.sum() == 30 { return 0; }
  return 1;
}
