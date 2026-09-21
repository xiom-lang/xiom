// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O06: Option[enum] -- Option wrapping a user-defined enum
enum Color { Red, Green, Blue(val: Int) }
fn main() -> Int {
  var a: Option[Color] = Some(Color.Red);
  var b: Option[Color] = Some(Color.Blue(42));
  var c: Option[Color] = None;
  match a { Some(v) => { match v { Red => {} _ => { return 1; } } } None => { return 2; } }
  match b { Some(v) => { match v { Blue(x) => { if x != 42 { return 3; } } _ => { return 4; } } } None => { return 5; } }
  match c { Some(_) => { return 6; } None => {} }
  return 0;
}
