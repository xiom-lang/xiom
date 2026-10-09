// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m243 (honest containment, follow-up to m241): FIXED-ARRAY `[N]T` index
// reads and writes must trap out of bounds under --overflow-checks, like
// Vec indexing already does. Pre-fix `a[100] = 42; a[100]` silently wrote
// and read the stack slot out of bounds (rc 42); the e2e lock compiles
// THIS file with --overflow-checks and asserts a NON-ZERO exit. Default
// (no-flag) builds keep the historical unchecked path.
module m243_fixed_array_bounds

fn main() -> Int {
  let a = [1, 2, 3];
  a[100] = 42;
  var sink = a[100];
  return sink;
}
