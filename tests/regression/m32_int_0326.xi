// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Bool-casting alternatives (no Bool->Int cast in XIOM)
fn main() -> Int {
  var b: Bool = true;
  var as_int: Int = if b { 1 } else { 0 };
  if as_int == 1 { return 0; }
  return 1;
}
