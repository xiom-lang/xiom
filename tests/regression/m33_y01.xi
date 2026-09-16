// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y01: struct + enum + generic + match + contract + method + module + diff
type NumPair = { a: Int; b: Int; } derive[Eq]
enum CalcKind { Add, Mul, Max }
fn compute[T](p: NumPair, k: CalcKind) -> Int
  requires: p.a >= 0
  requires: p.b >= 0
  ensures: result >= 0
{
  match k {
    CalcKind.Add => p.a + p.b,
    CalcKind.Mul => p.a * p.b,
    CalcKind.Max => if p.a > p.b { p.a } else { p.b },
  }
}
interface Summable { fn total(self) -> Int; }
impl Summable for NumPair {
  fn total(self) -> Int { return self.a + self.b; }
}
module ops {
  pub fn add_pair(p: NumPair) -> Int { return p.a + p.b; }
  pub fn via_impl(p: NumPair) -> Int { return p.total(); }
}
use ops.add_pair;
use ops.via_impl;
enum VerifyKind { Loop, Formula }
fn verify(v: VerifyKind, p: NumPair, k: CalcKind) -> Int {
  match v {
    Loop => compute(p, k),
    Formula => p.total(),
  }
}
fn main() -> Int {
  var p = NumPair{ a: 7; b: 3; };
  var r1 = verify(VerifyKind.Loop, p, CalcKind.Add);
  var r2 = verify(VerifyKind.Formula, p, CalcKind.Add);
  var r3 = verify(VerifyKind.Loop, p, CalcKind.Mul);
  if r1 == r2 && r1 == 10 && r3 == 21 { return 0; }
  return 1;
}
