// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0095

interface Doubles {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return 2; }
}

type T1 = {}

fn T1.a(self) -> Int { return 1; }

fn T1.b(self) -> Int { return 2; }


type T2 = {}

fn T2.a(&self) -> Int { return 10; }

fn T2.b(&self) -> Int { return 20; }

fn main() -> Int {
  var t1: T1 = T1{};
  var t2: T2 = T2{};
  if t1.a() != 1 { return 1; }
  if t1.b() != 2 { return 2; }
  if t2.a() != 10 { return 3; }
  if t2.b() != 20 { return 4; }
  return 0;
}
