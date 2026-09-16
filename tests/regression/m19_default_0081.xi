// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0081

interface Named {
  fn label(&self) -> Str { return name() + ":" + kind(); }
  fn name(&self) -> Str;
}

interface Typed {
  fn kind(&self) -> Str;
}

type Item = { n: Str; k: Str; }

fn Item.label(self) -> Str { return self.name() + ":" + self.kind(); }


fn Item.name(&self) -> Str { return n; }

fn Item.kind(&self) -> Str { return k; }

fn main() -> Int {
  var i: Item = Item{ n: "foo", k: "bar" };
  if i.label() == "foo:bar" { return 0; }
  return 1;
}
