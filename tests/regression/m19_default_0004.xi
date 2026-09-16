// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0004

interface MultiDefault {
  fn first(&self) -> Int { return 10; }
  fn second(&self) -> Str { return "ten"; }
  fn label(&self) -> Str;
}

type Item = { name: Str; }

fn Item.first(self) -> Int { return 10; }

fn Item.second(self) -> Str { return "ten"; }


fn Item.label(&self) -> Str { return name; }

fn main() -> Int {
  var i: Item = Item{ name: "test" };
  if i.label() == "test" && i.first() == 10 && i.second() == "ten" { return 0; }
  return 1;
}
