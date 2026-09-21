// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0122

interface Constants {
  fn is_always(&self) -> Bool { return true; }
  fn is_never(&self) -> Bool { return false; }
  fn label(&self) -> Str;
}

type Item = { tag: Str; }

fn Item.is_always(self) -> Bool { return true; }

fn Item.is_never(self) -> Bool { return false; }


fn Item.label(&self) -> Str { return tag; }

fn main() -> Int {
  var i: Item = Item{ tag: "x" };
  if i.label() != "x" { return 1; }
  if i.is_always() != true { return 2; }
  if i.is_never() != false { return 3; }
  return 0;
}
