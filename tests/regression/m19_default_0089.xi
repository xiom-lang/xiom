// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0089

interface OptionalGetter {
  fn try_get(&self) -> Option[Int] {
    var v = value();
    if v >= 0 { return Some(v); }
    return None;
  }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.value(&self) -> Int { return x; }

fn main() -> Int {
  var n: Num = Num{ x: 42 };
  var r = n.try_get();
  match r {
    Some(v) => if v == 42 { return 0; },
    None => return 2
  }
  return 1;
}
