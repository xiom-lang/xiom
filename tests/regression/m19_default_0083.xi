// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0083

interface Grader {
  fn grade(&self) -> Str {
    match score() {
      s if s >= 80 => "A",
      s if s >= 50 => "B",
      _ => "F"
    }
  }
  fn score(&self) -> Int;
}

type Student = { s: Int; }

fn Student.score(&self) -> Int { return s; }

fn main() -> Int {
  var a: Student = Student{ s: 90 };
  var b: Student = Student{ s: 75 };
  var c: Student = Student{ s: 30 };
  if a.grade() != "A" { return 1; }
  if b.grade() != "B" { return 2; }
  if c.grade() != "F" { return 3; }
  return 0;
}
