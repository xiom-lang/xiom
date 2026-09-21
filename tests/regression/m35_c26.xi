// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C26: if-let pattern -- match on Option as control flow (XIOM uses match, not if-let)
fn maybe_double(opt: Option[Int]) -> Int {
  match opt {
    Some(v) => v * 2,
    None => 0,
  }
}
fn maybe_add(a: Option[Int], b: Option[Int]) -> Int {
  var va: Int = 0;
  var vb: Int = 0;
  match a { Some(x) => { va = x; } None => {} }
  match b { Some(x) => { vb = x; } None => {} }
  return va + vb;
}
fn main() -> Int {
  if maybe_double(Some(5)) != 10 { return 1; }
  if maybe_double(None) != 0 { return 2; }
  if maybe_add(Some(3), Some(7)) != 10 { return 3; }
  if maybe_add(Some(3), None) != 3 { return 4; }
  if maybe_add(None, Some(7)) != 7 { return 5; }
  if maybe_add(None, None) != 0 { return 6; }
  return 0;
}
