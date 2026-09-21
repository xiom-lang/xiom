// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_030
type Score = { val: Int; grade: Int; }
fn main() -> Int {
  var s: Score = Score{ val: 0; grade: 0; };
  var threshold = 50;
  if threshold > 40 {
    s.val = 75;
    s.grade = 2;
  } else {
    s.val = 25;
    s.grade = 1;
  }
  if s.val == 75 && s.grade == 2 { return 0; }
  return 1;
  return 1;
}
