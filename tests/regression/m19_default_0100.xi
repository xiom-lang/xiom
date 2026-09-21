// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0100

interface PowerCheck {
  fn is_pow2(&self) -> Bool {
    var v = value();
    while v > 1 && v % 2 == 0 {
      v = v / 2;
    }
    return v == 1;
  }
  fn value(&self) -> Int;
}

type Num = { x: Int; }

fn Num.value(self) -> Int { return self.x; }
fn Num.is_pow2(self) -> Bool {
    var v = self.value();
    while v > 1 && v % 2 == 0 {
      v = v / 2;
    }
    return v == 1;
  }

fn main() -> Int {
  var p1: Num = Num{ x: 1 };
  var p2: Num = Num{ x: 2 };
  var p3: Num = Num{ x: 4 };
  var p4: Num = Num{ x: 8 };
  var p5: Num = Num{ x: 16 };
  var p6: Num = Num{ x: 32 };
  var np1: Num = Num{ x: 3 };
  var np2: Num = Num{ x: 6 };
  var np3: Num = Num{ x: 10 };
  if !p1.is_pow2() { return 1; }
  if !p2.is_pow2() { return 2; }
  if !p3.is_pow2() { return 3; }
  if !p4.is_pow2() { return 4; }
  if !p5.is_pow2() { return 5; }
  if !p6.is_pow2() { return 6; }
  if np1.is_pow2() { return 7; }
  if np2.is_pow2() { return 8; }
  if np3.is_pow2() { return 9; }
  return 0;
}
