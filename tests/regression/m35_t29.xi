// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T29: Mixed combinators -- all types combined in complex expressions
type Record = { b: Bool; i: Int; f: Float64; c: Char; s: Str; }
enum Shape { Circle(r: Float64), Square(s: Int), Label(t: Str) }
fn process(r: Record) -> Int {
  var score: Int = 0;
  if r.b { score = score + 10; }
  score = score + r.i;
  score = score + r.f as Int;
  if r.c == 'A' { score = score + 100; }
  score = score + r.s.len() as Int;
  return score;
}
fn shape_weight(s: Shape) -> Float64 {
  match s {
    Shape.Circle(r) => r * 3.14,
    Shape.Square(w) => w as Float64 * 4.0,
    Shape.Label(t) => t.len() as Float64,
  }
}
fn mixed_cond(b: Bool, v: Int) -> Int {
  if b { return v + 5; }
  return v - 5;
}
fn generic_record[T](r: Record) -> Bool { return r.b; }
fn main() -> Int {
  var r: Record = Record{ b: true; i: 42; f: 3.0; c: 'A'; s: "test"; };
  var s: Int = process(r);
  if s != 159 { return 1; }
  var c1: Shape = Shape.Circle(2.0);
  if shape_weight(c1) != 6.28 { return 2; }
  var s1: Shape = Shape.Square(3);
  if shape_weight(s1) != 12.0 { return 3; }
  var l1: Shape = Shape.Label("hi");
  if shape_weight(l1) != 2.0 { return 4; }
  if mixed_cond(true, 10) != 15 { return 5; }
  if mixed_cond(false, 10) != 5 { return 6; }
  if !generic_record(r) { return 7; }
  return 0;
}

