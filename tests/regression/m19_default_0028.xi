// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0028

interface A {
  fn desc(&self) -> Str { return "A"; }
  fn name(&self) -> Str;
}

interface B {
  fn desc(&self) -> Str { return "B"; }
  fn name(&self) -> Str;
}

type Entity = { tag: Str; }

fn Entity.name(&self) -> Str { return tag; }

fn Entity.desc(&self) -> Str { return "mine"; }

fn main() -> Int {
  var e: Entity = Entity{ tag: "X" };
  if e.name() == "X" && e.desc() == "mine" { return 0; }
  return 1;
}
