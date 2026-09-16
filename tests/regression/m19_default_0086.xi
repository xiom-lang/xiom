// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0086

interface Divider {
  fn div2(&self) -> Int {
    var v = value();
    if v < 0 { return -1; }
    return v / 2;
  }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var pos: Num = Num{ x: 10 };
  var neg: Num = Num{ x: -5 };
  var zero: Num = Num{ x: 0 };
  if pos.div2() != 5 { return 1; }
  if neg.div2() != -1 { return 2; }
  if zero.div2() != 0 { return 3; }
  return 0;
}
