// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M30: All primitive types combined in computation
type Record = { i: Int; f: Float64; b: Bool; c: Char; }
fn score(r: Record) -> Int {
  var s: Int = 0;
  if r.i > 0 { s = s + r.i; }
  if r.f > 0.0 { s = s + 1; }
  if r.b { s = s + 10; }
  if r.c == 'A' { s = s + 100; }
  return s;
}
fn main() -> Int {
  var r = Record{ i: 42; f: 3.14; b: true; c: 'A'; };
  var s = score(r);
  if s == 153 { return 0; }
  return 1;
}
