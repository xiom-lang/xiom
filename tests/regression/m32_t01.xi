// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T01: String literal -- declare and compare
fn main() -> Int {
  var s: Str = "hello";
  if s == "hello" { return 0; }
  return 1;
}
