// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O04: Option[Bool] create -- Some/None with boolean values
fn main() -> Int {
  var a = Some(true);
  var b = Some(false);
  var c: Option[Bool] = None;
  match a { Some(v) => { if v != true { return 1; } } None => { return 2; } }
  match b { Some(v) => { if v != false { return 3; } } None => { return 4; } }
  match c { Some(_) => { return 5; } None => {} }
  return 0;
}
