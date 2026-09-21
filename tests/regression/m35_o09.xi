// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O09: Option and_then -- match-based and_then chaining
fn safe_div(a: Int, b: Int) -> Option[Int] {
  if b == 0 { return None; }
  return Some(a / b);
}
fn opt_and_then(o: Option[Int], f: fn(Int) -> Option[Int]) -> Option[Int] {
  match o {
    Some(v) => f(v),
    None => None
  }
}
fn main() -> Int {
  var r = safe_div(10, 2);
  match r { Some(v) => { if v != 5 { return 1; } } None => { return 2; } }
  var r2 = opt_and_then(r, safe_div2);
  match r2 { Some(v) => { if v != 2 { return 3; } } None => { return 4; } }
  var r3 = opt_and_then(safe_div(10, 0), safe_div2);
  match r3 { Some(_) => { return 5; } None => {} }
  return 0;
}
fn safe_div2(a: Int) -> Option[Int] { return safe_div(a, 2); }
