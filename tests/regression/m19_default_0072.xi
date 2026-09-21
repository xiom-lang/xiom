// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0072

interface Multiplier {
  fn double(&self) -> Int { return value() * 2; }
  fn triple(&self) -> Int { return value() * 3; }
  fn sum_all(&self) -> Int { return value() + double() + triple(); }
  fn value(&self) -> Int;
}

type Number = { n: Int; }

fn Number.double(self) -> Int { return self.value() * 2; }

fn Number.triple(self) -> Int { return self.value() * 3; }

fn Number.sum_all(self) -> Int { return self.value() + self.double() + self.triple(); }


fn Number.value(&self) -> Int { return n; }

fn main() -> Int {
  var num: Number = Number{ n: 5 };
  if num.value() == 5 && num.double() == 10 && num.triple() == 15 && num.sum_all() == 30 { return 0; }
  return 1;
}
