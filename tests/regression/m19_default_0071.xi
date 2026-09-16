// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0071

interface Tagger {
  fn tagged(&self) -> Str { return "[" + key() + "]"; }
  fn key(&self) -> Str;
}

type Entry = { id: Str; }

fn Entry.tagged(self) -> Str { return "[" + self.key() + "]"; }


fn Entry.key(&self) -> Str { return id; }

fn main() -> Int {
  var e: Entry = Entry{ id: "KEY" };
  if e.key() == "KEY" && e.tagged() == "[KEY]" { return 0; }
  return 1;
}
