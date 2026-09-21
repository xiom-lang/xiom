// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E: brace-form module `module x { }` -- GAP-13 verification
// The parser handles brace-form modules correctly.
// Returns 0 on success.

module e2e_brace_module {
  fn add(a: Int, b: Int) -> Int {
    return a + b;
  }

  fn main() -> Int {
    let s = add(10, 25);
    if s == 35 {
      return 0;
    }
    return 1;
  }
}
