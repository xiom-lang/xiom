// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E01: Simple enum with 3 variants, basic match
enum Color { Red, Green, Blue }
fn main() -> Int {
  var c = Color.Red;
  match c {
    Red => {}
    Green => { return 1; }
    Blue => { return 1; }
  }
  var g = Color.Green;
  match g {
    Red => { return 1; }
    Green => {}
    Blue => { return 1; }
  }
  var b = Color.Blue;
  match b {
    Red => { return 1; }
    Green => { return 1; }
    Blue => {}
  }
  return 0;
}
