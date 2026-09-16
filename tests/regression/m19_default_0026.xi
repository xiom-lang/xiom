// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0026

interface Incrementable {
  fn inc(&self) -> Int { return value() + 1; }
  fn value(&self) -> Int;
}

type Counter = { count: Int; }

fn Counter.inc(self) -> Int { return self.value() + 1; }


fn Counter.value(&self) -> Int { return count; }

fn main() -> Int {
  var c: Counter = Counter{ count: 5 };
  if c.value() == 5 && c.inc() == 6 { return 0; }
  return 1;
}
