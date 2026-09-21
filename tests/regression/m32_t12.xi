// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T12: Escaped string -- newline and tab escape sequences
fn main() -> Int {
  var s: Str = "hello\nworld\t!";
  if s.len() == 13 { return 0; }
  return 1;
}
