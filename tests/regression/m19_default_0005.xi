// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0005

interface Versioned {
  fn version(&self) -> Int { return 1; }
  fn category(&self) -> Str { return "base"; }
  fn name(&self) -> Str;
}

type Product = { title: Str; }

fn Product.category(self) -> Str { return "base"; }


fn Product.name(&self) -> Str { return title; }

fn Product.version(&self) -> Int { return 2; }

fn main() -> Int {
  var p: Product = Product{ title: "gadget" };
  if p.name() == "gadget" && p.version() == 2 && p.category() == "base" { return 0; }
  return 1;
}
