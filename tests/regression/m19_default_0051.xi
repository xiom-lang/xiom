// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0051

interface Compute {
  fn compute(&self) -> Int {
    var a = value();
    var b = a * 2;
    var c = b + 10;
    return c;
  }
  fn value(&self) -> Int;
}

type Data = { val: Int; }

fn Data.compute(self) -> Int {
    var a = self.value();
    var b = a * 2;
    var c = b + 10;
    return c;
  }


fn Data.value(&self) -> Int { return val; }

fn main() -> Int {
  var d: Data = Data{ val: 7 };
  if d.value() == 7 && d.compute() == 24 { return 0; }
  return 1;
}
