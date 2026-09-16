// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E12: Very short names -- single-char identifiers
fn f(a: Int, b: Int, c: Int) -> Int { return a + b + c; }
fn main() -> Int {
  var x = 1; var y = 2; var z = 3;
  var r = f(x, y, z);
  if r != 6 { return 1; }
  return 0;
}
