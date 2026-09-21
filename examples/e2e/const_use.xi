// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E regression: pub const referenced across functions (GAP-3). Returns 0.
module e2e_const_use

pub const LIMIT: Int = 100;

fn under(x: Int) -> Bool { return x < LIMIT; }

fn main() -> Int {
  if under(50) && LIMIT == 100 { return 0; }
  return 1;
}
