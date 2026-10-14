// m311 lock (COMPILER_BUGS m283 finding): a violated clause whose condition
// constant-folds after inlining (`requires: x != x` on a constant input)
// must abort on OPTIMIZED builds. clang -O2 previously folded the
// post-inline comparison and pruned the violated-clause path; the volatile
// no-fold barrier in compile_contract_check keeps the check's branch
// runtime-decided. Expected: abort (exit 1 + `contract violated`) at
// --opt-level 0 AND --opt-level 2.
// Pure XIOM. Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m311_o2_contract_barrier

use xiom.io;

fn never_ok(x: Int) -> Int
  requires: x != x
{
  return x;
}

fn main() -> Int {
  io.println("before");
  let v = never_ok(5);
  io.println("UNREACHABLE v=" + to_string(v));
  return 0;
}
