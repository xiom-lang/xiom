// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn opt_filter(o: Option[Int], pred: fn(Int) -> Bool) -> Option[Int] {
  match o { Some(v) => { if pred(v) { Some(v) } else { None } } None => None }
}
fn pos(x: Int) -> Bool { return x > 10; }
fn main() -> Int {
  match opt_filter(Some(42), pos) { Some(v) => { if v != 42 { return 1; } } None => { return 2; } }
  match opt_filter(Some(3), pos) { Some(_) => { return 3; } None => {} }
  match opt_filter(None, pos) { Some(_) => { return 4; } None => {} }
  return 0;
}
