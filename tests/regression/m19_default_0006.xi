// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0006

interface Tagger {
  fn summary(&self) -> Str { return name() + ":"; }
  fn name(&self) -> Str;
}

type Tag = { value: Str; }

fn Tag.summary(self) -> Str { return self.name() + ":"; }


fn Tag.name(&self) -> Str { return value; }

fn main() -> Int {
  var t: Tag = Tag{ value: "urgent" };
  if t.name() == "urgent" && t.summary() == "urgent:" { return 0; }
  return 1;
}
