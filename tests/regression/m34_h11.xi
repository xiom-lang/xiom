// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H11: Bool->Int pattern -- true evaluates as non-zero, false as zero
fn bool_to_int(b: Bool) -> Int { if b { return 1; } return 0; }
fn main() -> Int {
  if bool_to_int(true) == 1 && bool_to_int(false) == 0 { return 0; }
  return 1;
}
