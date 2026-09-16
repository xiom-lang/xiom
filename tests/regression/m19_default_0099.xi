// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0099

interface Revealable {
  fn reveal(&self) -> Int { return get() + 10; }
  fn get(&self) -> Int;
}

type Data = { secret: Int; }

fn Data.reveal(self) -> Int { return self.get() + 10; }


fn Data.get(&self) -> Int { return secret; }

fn main() -> Int {
  var d: Data = Data{ secret: 42 };
  if d.get() == 42 && d.reveal() == 52 { return 0; }
  return 1;
}
