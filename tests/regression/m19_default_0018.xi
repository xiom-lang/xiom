// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0018

interface Taggable {
  fn describe(&self) -> Str { return "enum"; }
  fn kind(&self) -> Str;
}

enum Color {
  Red,
  Green,
  Blue
}

fn Color.kind(self) -> Str { return "color"; }
fn Color.describe(self) -> Str { return "enum"; }

fn main() -> Int {
  var c: Color = Color.Red;
  if c.kind() == "color" && c.describe() == "enum" { return 0; }
  return 1;
}
