// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O07: Option unwrap_or -- match-based unwrap_or implementation
fn unwrap_or_int(o: Option[Int], default: Int) -> Int {
  match o { Some(v) => { return v; } None => { return default; } }
}
fn main() -> Int {
  var a = Some(42);
  var b: Option[Int] = None;
  if unwrap_or_int(a, 0) != 42 { return 1; }
  if unwrap_or_int(b, 99) != 99 { return 2; }
  if unwrap_or_int(Some(-5), 10) != -5 { return 3; }
  if unwrap_or_int(None, 7) != 7 { return 4; }
  return 0;
}
