// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0062

interface Total {
  fn total(&self) -> Int { return a() + b() + c(); }
  fn a(&self) -> Int;
  fn b(&self) -> Int;
  fn c(&self) -> Int;
}

type Triple = { x: Int; y: Int; z: Int; }

fn Triple.total(self) -> Int { return self.a() + self.b() + self.c(); }


fn Triple.a(&self) -> Int { return x; }

fn Triple.b(&self) -> Int { return y; }

fn Triple.c(&self) -> Int { return z; }

fn main() -> Int {
  var t: Triple = Triple{ x: 1, y: 2, z: 3 };
  if t.a() == 1 && t.b() == 2 && t.c() == 3 && t.total() == 6 { return 0; }
  return 1;
}
