// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0076

enum Color { Red, Green, Blue }

fn main() -> Int {
  var c: Color = Green;
  match c {
    Red | Green if true => { return 0; }
    _ => { return 1; }
  }
}
