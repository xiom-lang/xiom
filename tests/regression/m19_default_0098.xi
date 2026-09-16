// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0098

interface BigInterface {
  fn sum(&self) -> Int { return a() + b() + c() + d() + e(); }
  fn prod(&self) -> Int { return a() * b() * c() * d() * e(); }
  fn avg(&self) -> Int { return sum() / 5; }
  fn min_val(&self) -> Int {
    var s = a();
    if b() < s { s = b(); }
    if c() < s { s = c(); }
    if d() < s { s = d(); }
    if e() < s { s = e(); }
    return s;
  }
  fn max_val(&self) -> Int {
    var m = a();
    if b() > m { m = b(); }
    if c() > m { m = c(); }
    if d() > m { m = d(); }
    if e() > m { m = e(); }
    return m;
  }
  fn a(&self) -> Int;
  fn b(&self) -> Int;
  fn c(&self) -> Int;
  fn d(&self) -> Int;
  fn e(&self) -> Int;
}

type Five = { v1: Int; v2: Int; v3: Int; v4: Int; v5: Int; }

fn Five.a(&self) -> Int { return v1; }

fn Five.b(&self) -> Int { return v2; }

fn Five.c(&self) -> Int { return v3; }

fn Five.d(&self) -> Int { return v4; }

fn Five.e(&self) -> Int { return v5; }

fn main() -> Int {
  var f: Five = Five{ v1: 1, v2: 2, v3: 3, v4: 4, v5: 5 };
  if f.sum() != 15 { return 1; }
  if f.prod() != 120 { return 2; }
  if f.avg() != 3 { return 3; }
  if f.min_val() != 1 { return 4; }
  if f.max_val() != 5 { return 5; }
  return 0;
}
