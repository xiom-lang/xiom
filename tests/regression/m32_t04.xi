// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T04: String length -- .len() returns Int
fn main() -> Int {
  var s: Str = "hello";
  var n: Int = s.len();
  if n == 5 { return 0; }
  return 1;
}
