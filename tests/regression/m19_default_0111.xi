// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0111

interface Grading {
  fn grade(&self) -> Str { var v = score(); if v >= 90 { return "A"; } elif v >= 80 { return "B"; } elif v >= 70 { return "C"; } else { return "F"; } }
  fn score(&self) -> Int;
}

type Student = { points: Int; }

fn Student.score(&self) -> Int { return points; }

fn main() -> Int {
  var s: Student = Student{ points: 85 };
  if s.score() != 85 { return 1; }
  if s.grade() != "B" { return 2; }
  return 0;
}
