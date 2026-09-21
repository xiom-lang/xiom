// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0117

interface Accessor {
  fn clone_val(&self) -> Int { return value(); }
  fn value(&self) -> Int;
}

type Cell = { x: Int; }

fn Cell.clone_val(self) -> Int { return self.value(); }


fn Cell.value(&self) -> Int { return x; }

fn main() -> Int {
  var c: Cell = Cell{ x: 77 };
  if c.value() != 77 { return 1; }
  if c.clone_val() != 77 { return 2; }
  return 0;
}
