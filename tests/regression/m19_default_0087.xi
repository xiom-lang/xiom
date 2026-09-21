// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0087

interface Computer {
  fn compute(&self) -> Int {
    var x = value();
    var y = x + 1;
    var z = y * 2;
    return z;
  }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.compute(self) -> Int {
    var x = self.value();
    var y = x + 1;
    var z = y * 2;
    return z;
  }


fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 5 };
  if n.value() == 5 && n.compute() == 12 { return 0; }
  return 1;
}
