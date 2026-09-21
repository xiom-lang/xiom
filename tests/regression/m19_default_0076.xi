// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0076

interface Chain {
  fn double(&self) -> Int { return value() * 2; }
  fn quadruple(&self) -> Int { return double() * 2; }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.double(self) -> Int { return self.value() * 2; }

fn Num.quadruple(self) -> Int { return self.double() * 2; }


fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 5 };
  if n.value() == 5 && n.double() == 10 && n.quadruple() == 20 { return 0; }
  return 1;
}
