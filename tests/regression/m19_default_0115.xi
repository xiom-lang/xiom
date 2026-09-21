// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0115

interface Chain {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return a() * 2; }
  fn c(&self) -> Int { return b() * 3; }
  fn d(&self) -> Int { return c() * 4; }
  fn e(&self) -> Int { return d() * 5; }
}

type Node = {}

fn Node.a(self) -> Int { return 1; }

fn Node.b(self) -> Int { return self.a() * 2; }

fn Node.c(self) -> Int { return self.b() * 3; }

fn Node.d(self) -> Int { return self.c() * 4; }

fn Node.e(self) -> Int { return self.d() * 5; }


fn main() -> Int {
  var n: Node = Node{};
  if n.a() != 1 { return 1; }
  if n.b() != 2 { return 2; }
  if n.c() != 6 { return 3; }
  if n.d() != 24 { return 4; }
  if n.e() != 120 { return 5; }
  return 0;
}
