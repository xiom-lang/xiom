// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0097

interface Arithmetic {
  fn calc(&self) -> Int { return a() + b() * c() - d() / e() + f() % g(); }
  fn a(&self) -> Int;
  fn b(&self) -> Int;
  fn c(&self) -> Int;
  fn d(&self) -> Int;
  fn e(&self) -> Int;
  fn f(&self) -> Int;
  fn g(&self) -> Int;
}

type Values = { v1: Int; v2: Int; v3: Int; v4: Int; v5: Int; v6: Int; v7: Int; }

fn Values.calc(self) -> Int { return self.a() + self.b() * self.c() - self.d() / self.e() + self.f() % self.g(); }


fn Values.a(&self) -> Int { return v1; }

fn Values.b(&self) -> Int { return v2; }

fn Values.c(&self) -> Int { return v3; }

fn Values.d(&self) -> Int { return v4; }

fn Values.e(&self) -> Int { return v5; }

fn Values.f(&self) -> Int { return v6; }

fn Values.g(&self) -> Int { return v7; }

fn main() -> Int {
  var v: Values = Values{ v1: 1, v2: 2, v3: 3, v4: 4, v5: 2, v6: 5, v7: 3 };
  var result = v.calc();
  if result == 1 + 2 * 3 - 4 / 2 + 5 % 3 { return 0; }
  return 1;
}
