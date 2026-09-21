// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0032

interface OptionalValue {
  fn try_value(&self) -> Option[Int] {
    var v = value();
    if v >= 0 { return Some(v); }
    return None;
  }
  fn value(&self) -> Int;
}

type Source = { val: Int; }

fn Source.value(&self) -> Int { return val; }

fn main() -> Int {
  var pos: Source = Source{ val: 7 };
  var neg: Source = Source{ val: -3 };
  var p = pos.try_value();
  var n = neg.try_value();
  match p {
    Some(v) => if v != 7 { return 1; },
    None => return 2
  }
  match n {
    Some(_) => return 3,
    None => { }
  }
  return 0;
}
