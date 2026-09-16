// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0041

interface Dividable {
  fn safe_div(&self) -> Int {
    var d = divisor();
    if d == 0 { return 0; }
    return value() / d;
  }
  fn value(&self) -> Int;
  fn divisor(&self) -> Int;
}

type Fraction = { num: Int; den: Int; }

fn Fraction.value(&self) -> Int { return num; }

fn Fraction.divisor(&self) -> Int { return den; }

fn main() -> Int {
  var ok: Fraction = Fraction{ num: 12, den: 4 };
  var byzero: Fraction = Fraction{ num: 12, den: 0 };
  if ok.safe_div() != 3 { return 1; }
  if byzero.safe_div() != 0 { return 2; }
  return 0;
}
