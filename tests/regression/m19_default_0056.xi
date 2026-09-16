// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0056

interface Taggable {
  fn tag(&self) -> Char { return '?'; }
  fn kind(&self) -> Int;
}

type Data = { k: Int; }

fn Data.tag(self) -> Char { return '?'; }


fn Data.kind(&self) -> Int { return k; }

fn main() -> Int {
  var d: Data = Data{ k: 42 };
  if d.kind() == 42 && d.tag() == '?' { return 0; }
  return 1;
}
