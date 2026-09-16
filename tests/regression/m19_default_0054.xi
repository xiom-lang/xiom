// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0054

interface Defaults {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return 2; }
}

type Empty = {}

fn Empty.a(self) -> Int { return 1; }

fn Empty.b(self) -> Int { return 2; }


fn main() -> Int {
  var e: Empty = Empty{};
  if e.a() == 1 && e.b() == 2 { return 0; }
  return 1;
}
