// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O11: Option or_else -- match-based or_else fallback
fn opt_or_else(o: Option[Int], fallback: fn() -> Option[Int]) -> Option[Int] {
  match o { Some(v) => Some(v), None => fallback() }
}
fn some_seven() -> Option[Int] { return Some(7); }
fn none() -> Option[Int] { return None; }
fn main() -> Int {
  match opt_or_else(Some(42), some_seven) { Some(v) => { if v != 42 { return 1; } } None => { return 2; } }
  match opt_or_else(None, some_seven) { Some(v) => { if v != 7 { return 3; } } None => { return 4; } }
  match opt_or_else(None, none) { Some(_) => { return 5; } None => {} }
  match opt_or_else(Some(0), none) { Some(v) => { if v != 0 { return 6; } } None => { return 7; } }
  return 0;
}
