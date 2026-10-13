// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m291 lock (REL-20261010-1548-stdlib-7 enforcement half): a DIRECTLY
// violated `requires` must abort at runtime. The m283 contract re-entrancy
// guard skipped every clause check (skip test `d > 0` while the counter is
// incremented at function entry, so d == 1 at the first clause site):
// runtime requires/ensures enforcement was silently dead and this program
// returned 0. The guard now skips only a RE-ENTRANT evaluation (d > 1).
// The checker_locks test asserts a NONZERO run exit + the violation message.
module m291_contract_enforcement.main

fn f(x: Int) -> Int
  requires: x > 0
{
  return x + 1;
}

fn main() -> Int {
  let v = f(0);
  if v != 1 { return 2; }
  return 0;
}
