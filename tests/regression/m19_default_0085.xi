// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0085

interface Counter {
  fn countdown(&self) -> Int {
    var n = value();
    var steps = 0;
    while n > 0 {
      n = n - 1;
      steps = steps + 1;
    }
    return steps;
  }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 5 };
  if n.value() == 5 && n.countdown() == 5 { return 0; }
  return 1;
}
