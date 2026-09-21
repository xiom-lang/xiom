// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

interface Ident { fn id(self) -> Int; }
type W = { n: Int; }
impl Ident for W {
  fn id(self) -> Int { return self.n; }
}
fn main() -> Int {
  var w = W{ n: 99 };
  if w.id() != 99 { return 1; }
  return 0;
}