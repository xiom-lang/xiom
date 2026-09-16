// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// Inline Assembly Tests -- v0.55
// Verifies asm("nop") compiles and runs correctly.
// Returns 0 on success.

fn main() -> Int {
  // Basic: nop -- does nothing, just verifies asm compiles and links
  unsafe { asm("nop"); }

  // Verify return reaches here
  return 0;
}
