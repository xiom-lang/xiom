// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T07: Empty string -- length and comparison
fn main() -> Int {
  var e: Str = "";
  if e.len() == 0 && e == "" { return 0; }
  return 1;
}
