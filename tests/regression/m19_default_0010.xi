// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0010

interface Displayable {
  fn display(&self) -> Str { return "[" + label() + "]"; }
  fn label(&self) -> Str;
}

type Box = { text: Str; }

fn Box.display(self) -> Str { return "[" + self.label() + "]"; }


fn Box.label(&self) -> Str { return text; }

fn main() -> Int {
  var b: Box = Box{ text: "contents" };
  if b.label() == "contents" && b.display() == "[contents]" { return 0; }
  return 1;
}
