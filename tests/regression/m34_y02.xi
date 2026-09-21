// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y02: nested if-else chain inside match arm + compound assign + module + derive
type Score = { points: Int; multiplier: Int; } derive[Eq]
enum Op { Double, Triple, Reset }
fn do_double(s: Score) -> Int {
  var m = s.multiplier;
  var r = s.points;
  r = r + m;
  r = r * 2;
  return r;
}
fn do_triple(s: Score) -> Int {
  var m = s.multiplier;
  var r = s.points;
  if m > 1 { r = r * 3; m = m - 1; r = r + m; }
  else { if m == 1 { r = r * 3; } else { r = 0; } }
  return r;
}
fn compute[T](s: Score, op: Op) -> Int {
  match op {
    Double => do_double(s),
    Triple => do_triple(s),
    Reset => 0,
  }
}
module calc {
  pub fn do_compute(s: Score, op: Op) -> Int { return compute(s, op); }
  pub fn sum(s: Score) -> Int { return s.points * s.multiplier; }
}
use calc.do_compute;
use calc.sum;
fn main() -> Int {
  var s = Score{ points: 10; multiplier: 2; };
  var r1 = do_compute(s, Op.Double);
  var r2 = do_compute(s, Op.Triple);
  if r1 == 24 && r2 == 31 { return 0; }
  return 1;
}
