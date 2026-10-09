// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m241 (honest containment, t8 arena): an out-of-bounds Vec INDEX WRITE
// must trap under --overflow-checks, exactly like the read path already
// does. Pre-fix `buf[100] = 42` was silent UB (SILENT_UB in the arena
// while the read probe was RUNTIME_PANIC), and the write corrupted
// adjacent heap bytes without any signal.
//
// The e2e lock compiles THIS file with --overflow-checks and asserts a
// NON-ZERO exit (llvm.trap). Under default flags (no overflow checks) the
// historical unchecked behavior is unchanged.
module m241_vec_write_bounds

fn main() -> Int {
  var buf = Vec[Int].with_capacity(4);
  buf.push(1); buf.push(2); buf.push(3); buf.push(4);
  buf[100] = 42;
  return 0;
}
