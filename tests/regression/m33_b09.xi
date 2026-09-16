// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B09: Let binding copies -- new let reads original, both independent
fn main() -> Int {
  let a = 33;
  let b = a;
  let c = b + a;
  if a == 33 && b == 33 && c == 66 { return 0; }
  return 1;
}
