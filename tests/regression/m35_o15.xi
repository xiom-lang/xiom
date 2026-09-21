// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O15: Option as_ref pattern -- pass Option by reference, match on &Option
fn examine(o: &Option[Int]) -> Int {
  match o {
    Some(v) => { return v * 2; }
    None => { return -1; }
  }
}
fn main() -> Int {
  var a = Some(21);
  var b: Option[Int] = None;
  if examine(&a) != 42 { return 1; }
  if examine(&b) != -1 { return 2; }
  var c = Some(0);
  if examine(&c) != 0 { return 3; }
  return 0;
}
