// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N09: Enum with derive[Ord] -- ordinal position comparison via rank function
enum Priority { Low, Medium, High, Critical } derive[Ord, Eq]
fn rank(p: Priority) -> Int {
  match p {
    Priority.Low => 0,
    Priority.Medium => 1,
    Priority.High => 2,
    Priority.Critical => 3,
  }
}
fn main() -> Int {
  var a = Priority.Low;
  var b = Priority.High;
  var c = Priority.Medium;
  var d = Priority.Medium;
  if rank(a) < rank(b) && rank(b) > rank(c) && c == d { return 0; }
  return 1;
}
