// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0012

interface Doubler {
  fn double(&self) -> Int { return value() * 2; }
  fn value(&self) -> Int;
}

type Number = { n: Int; }

fn Number.double(self) -> Int { return self.value() * 2; }


fn Number.value(&self) -> Int { return n; }

fn main() -> Int {
  var num: Number = Number{ n: 7 };
  if num.value() == 7 && num.double() == 14 { return 0; }
  return 1;
}
