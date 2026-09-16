// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H10: Int->Char -- integer codepoint to character
fn main() -> Int {
  var i: Int = 66;
  var c: Char = i as Char;
  if c == 'B' { return 0; }
  return 1;
}
