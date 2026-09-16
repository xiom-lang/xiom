// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H09: Char->Int -- character to its integer codepoint
fn main() -> Int {
  var c: Char = 'A';
  var i: Int = c as Int;
  if i == 65 { return 0; }
  return 1;
}
