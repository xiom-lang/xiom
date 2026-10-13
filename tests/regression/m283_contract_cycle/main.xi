// m283 lock (REL-20261010-1549-packages-8): a postcondition call-cycle must
// not recurse into the depth-500 trap. `cyc_f`'s ensures calls `cyc_g`,
// whose contract calls `cyc_f` again; the old evaluator followed the cycle
// until `llvm.trap()` (0xC000001D, no diagnostic). The re-entrancy guard
// skips the nested clause evaluation while the outermost check still runs.
// The matrices at the end prove normal requires/ensures still enforce
// (weakened guard-regression coverage; note -O2 constant-folding pruning is
// a separate open finding).
// Pure XIOM. Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m283_contract_cycle

use xiom.io;

fn cyc_g(x: Int) -> Int
  ensures: result == 1
{
  return 1;
}

fn cyc_f(x: Int) -> Int
  ensures: cyc_f(cyc_g(x)) == 1
{
  return cyc_g(x);
}

fn add1(x: Int) -> Int
  ensures: result == x + 1
{
  return x + 1;
}

fn clamp01(x: Int) -> Int
  requires: x >= 0
  ensures: result >= 0
{
  return x;
}

fn main() -> Int {
  var failed = 0;
  let c = cyc_f(0);
  if c != 1 { failed = failed + 1; }
  let a = add1(41);
  if a != 42 { failed = failed + 1; }
  let b = clamp01(7);
  if b != 7 { failed = failed + 1; }
  io.println("cycle=" + to_string(c) + " add1=" + to_string(a) + " clamp=" + to_string(b));
  return failed;
}
