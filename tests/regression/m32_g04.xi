// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G04: Generic enum with payload
fn main() -> Int {
  var a = Some(42);
  match a {
    Some(v) => { if v != 42 { return 1; } }
    None => { return 2; }
  }
  var b: Option[Int] = None;
  match b {
    Some(_) => { return 3; }
    None => {}
  }
  return 0;
}
