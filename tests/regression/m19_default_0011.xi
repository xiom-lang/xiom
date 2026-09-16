// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0011

interface Gradable {
  fn grade(&self) -> Str {
    var s = score();
    if s >= 80 { return "A"; }
    return "F";
  }
  fn score(&self) -> Int;
}

type Student = { points: Int; }

fn Student.score(&self) -> Int { return points; }

fn main() -> Int {
  var a: Student = Student{ points: 85 };
  var f: Student = Student{ points: 42 };
  if a.grade() == "A" && f.grade() == "F" { return 0; }
  return 1;
}
