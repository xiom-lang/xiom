// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0042

interface Scalable {
  fn double(&self) -> Int { return value() * 2; }
  fn quadruple(&self) -> Int { return double() * 2; }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.double(self) -> Int { return self.value() * 2; }

fn Num.quadruple(self) -> Int { return self.double() * 2; }


fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 3 };
  if n.value() == 3 && n.double() == 6 && n.quadruple() == 12 { return 0; }
  return 1;
}
