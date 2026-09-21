// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X03: Combinatorial + Differential -- if-chain vs match with generic enum dispatch
enum Grade { A, B, C, D, F }
fn classify_if(score: Int) -> Grade {
  if score >= 90 { return Grade.A; }
  elif score >= 80 { return Grade.B; }
  elif score >= 70 { return Grade.C; }
  elif score >= 60 { return Grade.D; }
  return Grade.F;
}
fn classify_match(score: Int) -> Grade {
  if score >= 90 { return Grade.A; }
  elif score >= 80 { return Grade.B; }
  elif score >= 70 { return Grade.C; }
  elif score >= 60 { return Grade.D; }
  return Grade.F;
}
fn grade_to_val(g: Grade) -> Int {
  match g { A => 4, B => 3, C => 2, D => 1, F => 0, }
}
fn main() -> Int {
  var r1 = grade_to_val(classify_if(95));
  var r2 = grade_to_val(classify_match(95));
  var r3 = grade_to_val(classify_if(72));
  var r4 = grade_to_val(classify_match(72));
  var r5 = grade_to_val(classify_if(55));
  var r6 = grade_to_val(classify_match(55));
  if r1 == r2 && r3 == r4 && r5 == r6 { return 0; }
  return 1;
}
