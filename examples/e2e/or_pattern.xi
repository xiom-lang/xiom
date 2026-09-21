// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E: or-pattern in match expression
// Verifies that `Pattern::Or` compiles correctly (Phase 4A).
// Returns 0 on success.

module e2e_or_pattern

fn classify(n: Int) -> Int {
  match n {
    1 | 2 | 3 => 10,
    4 | 5 | 6 => 20,
    _ => 0,
  }
}

fn main() -> Int {
  let a = classify(1);
  let b = classify(5);
  let c = classify(7);
  if a == 10 && b == 20 && c == 0 {
    return 0;
  }
  return 1;
}
