// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// Combinatorial stress: all types in one function
type Record = { i: Int; f: Float64; b: Bool; c: Char; s: Str; }
fn compute(r: Record) -> Int {
  var score: Int = 0;
  if r.i > 0 { score = score + r.i; }
  if r.f > 0.0 { score = score + 1; }
  if r.b { score = score + 10; }
  if r.c == 'A' { score = score + 100; }
  score = score + r.s.len() as Int;
  return score;
}
fn main() -> Int {
  var r = Record{ i: 42; f: 3.14; b: true; c: 'A'; s: "test"; };
  var s = compute(r);
  if s == 157 { return 0; }
  return 1;
}
