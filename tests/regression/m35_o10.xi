// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O10: Option filter -- match-based filter implementation
fn opt_filter(o: Option[Int], pred: fn(Int) -> Bool) -> Option[Int] {
  match o {
    Some(v) => { if pred(v) { return Some(v); } return None; }
    None => None
  }
}
fn is_even(x: Int) -> Bool { return x % 2 == 0; }
fn is_positive(x: Int) -> Bool { return x > 0; }
fn main() -> Int {
  match opt_filter(Some(4), is_even) { Some(v) => { if v != 4 { return 1; } } None => { return 2; } }
  match opt_filter(Some(3), is_even) { Some(_) => { return 3; } None => {} }
  match opt_filter(None, is_even) { Some(_) => { return 4; } None => {} }
  match opt_filter(Some(-1), is_positive) { Some(_) => { return 5; } None => {} }
  match opt_filter(Some(10), is_positive) { Some(v) => { if v != 10 { return 6; } } None => { return 7; } }
  return 0;
}
