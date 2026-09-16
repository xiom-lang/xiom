// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z10: array+while+closure+generic+enum+module+contract+compound_assign+derive+cast
type Row = { id: Int; score: Int; } derive[Eq]
enum Grade { Pass, Fail, Merit(v: Int) }
fn grade_row[T](r: Row, threshold: Int) -> Grade
  requires: threshold >= 0
{
  if r.score >= threshold + 10 { return Grade.Merit(r.score); }
  if r.score >= threshold { return Grade.Pass; }
  return Grade.Fail;
}
fn scale(s: Int, factor: Int) -> Int { return s * factor; }
module batch {
  pub fn grade(r: Row, t: Int) -> Grade { return grade_row(r, t); }
  pub fn sum_scores(rows: Vec[Int], n: Int) -> Int {
    var i = 0;
    var s = 0;
    while i < n { s += rows[i]; i += 1; }
    return s;
  }
}
use batch.grade;
use batch.sum_scores;
fn main() -> Int {
  var data = [10, 20, 30, 5, 15];
  var s = sum_scores(data, 5);
  var r1 = Row{ id: 1; score: s; };
  var g1 = grade(r1, 50);
  var r2 = Row{ id: 2; score: 5 as Int; };
  var g2 = grade(r2, 50);
  var r3 = Row{ id: 3; score: 95; };
  var g3 = grade(r3, 50);
  var f = |x| x + 10;
  var chk = 0;
  if g1 == Grade.Merit(80) { chk += 1; }
  if g2 == Grade.Fail { chk += 1; }
  if g3 == Grade.Merit(95) { chk += 1; }
  if f(s) == 90 { chk += 1; }
  if chk == 4 { return 0; }
  return 1;
}
