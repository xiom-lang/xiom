// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0021

interface TripleDefault {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Str { return "B"; }
  fn c(&self) -> Bool { return true; }
  fn d(&self) -> Str;
}

type Triple = { data: Str; }

fn Triple.a(self) -> Int { return 1; }

fn Triple.b(self) -> Str { return "B"; }

fn Triple.c(self) -> Bool { return true; }


fn Triple.d(&self) -> Str { return data; }

fn main() -> Int {
  var t: Triple = Triple{ data: "D" };
  if t.a() == 1 && t.b() == "B" && t.c() && t.d() == "D" { return 0; }
  return 1;
}
