// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0053

interface Cloneable {
  fn clone_value(&self) -> Int { return value(); }
  fn value(&self) -> Int;
}

type Number = { n: Int; }

fn Number.clone_value(self) -> Int { return self.value(); }


fn Number.value(&self) -> Int { return n; }

fn main() -> Int {
  var num: Number = Number{ n: 99 };
  if num.value() == 99 && num.clone_value() == 99 { return 0; }
  return 1;
}
