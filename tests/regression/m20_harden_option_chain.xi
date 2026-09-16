// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn div(a: Int, b: Int) -> Option[Int] {
  if b == 0 { return None; }
  return Some(a / b);
}
fn main() -> Int {
  match div(10, 2) {
    Some(v) => { if v != 5 { return 1; } }
    None => { return 2; }
  }
  match div(10, 0) {
    Some(_) => { return 3; }
    None => {}
  }
  return 0;
}