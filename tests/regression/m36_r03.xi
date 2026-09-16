// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-R03: extra closing brace -- brace-depth stress with if-true nesting
fn main() -> Int {
  if true { if true { if true { return 0; } } };
  return 1;
}
