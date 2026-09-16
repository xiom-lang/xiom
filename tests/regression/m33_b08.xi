// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B08: Clone to duplicate -- .clone() makes independent copy for Int-like types
fn main() -> Int {
  var a = 100;
  var b = a;
  var c = b;
  if a == 100 && b == 100 && c == 100 { return 0; }
  return 1;
}
