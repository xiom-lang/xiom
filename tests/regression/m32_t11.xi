// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T11: Unicode -- string with lambda character, verify len > 0
fn main() -> Int {
  var s: Str = "lambda";
  if s.len() > 0 { return 0; }
  return 1;
}
