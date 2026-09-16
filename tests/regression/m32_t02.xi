// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T02: Char literal -- declare and cast to Int
fn main() -> Int {
  var c: Char = 'A';
  if c as Int == 65 { return 0; }
  return 1;
}
