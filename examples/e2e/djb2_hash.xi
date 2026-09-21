// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E: DJB2 hash computation via generic Hash interface
// Verifies that hash[T: Hash](value: T) monomorphizes correctly
// and Int.hash produces deterministic DJB2 output.
// Returns 0 on success.

module e2e_djb2_hash

use xiom.hash;

fn main() -> Int {
  let h1 = hash.hash(42);
  let h2 = hash.hash(42);
  let h3 = hash.hash(43);
  // Same input -> same hash
  if h1 != h2 { return 1; }
  // Different inputs -> different hashes
  if h1 == h3 { return 1; }
  // Hash must be non-zero
  if h1 == 0 { return 1; }
  return 0;
}
