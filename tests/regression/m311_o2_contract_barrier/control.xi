// m311 control: constant-foldable clauses that HOLD must stay green at
// --opt-level 2 -- the no-fold barrier must never turn a satisfied clause
// into a spurious failure (nor block normal inlining/codegen of the body).
// Expected: exit 0, `control ok a=42` at --opt-level 0 and 2.
// Pure XIOM. Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m311_o2_contract_control

use xiom.io;

fn inc(x: Int) -> Int
  requires: x >= 0
  ensures: result == x + 1
{
  return x + 1;
}

fn main() -> Int {
  let a = inc(41);
  if a != 42 {
    io.println("FAIL a=" + to_string(a));
    return 1;
  }
  io.println("control ok a=" + to_string(a));
  return 0;
}
