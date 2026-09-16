// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0124

interface A {
  fn a(&self) -> Int { return 1; }
  fn name(&self) -> Str;
}

interface B {
  fn b(&self) -> Int { return 2; }
  fn name(&self) -> Str;
}

interface C {
  fn c(&self) -> Int { return 3; }
  fn name(&self) -> Str;
}

type Type1 = { tag: Str; }

fn Type1.b(self) -> Int { return 2; }

fn Type1.c(self) -> Int { return 3; }


fn Type1.name(&self) -> Str { return tag; }
fn Type1.a(&self) -> Int { return 10; }

type Type2 = { tag: Str; }

fn Type2.name(&self) -> Str { return tag; }

fn main() -> Int {
  var t1: Type1 = Type1{ tag: "first" };
  var t2: Type2 = Type2{ tag: "second" };
  if t1.name() != "first" { return 1; }
  if t1.a() != 10  { return 2; }
  if t1.b() != 2   { return 3; }
  if t1.c() != 3   { return 4; }
  if t2.name() != "second" { return 5; }
  if t2.a() != 1  { return 6; }
  if t2.b() != 2  { return 7; }
  if t2.c() != 3  { return 8; }
  return 0;
}
