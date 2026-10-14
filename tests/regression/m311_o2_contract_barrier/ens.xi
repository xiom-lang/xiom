// m311 lock: the ensures half of the constant-fold finding -- a violated
// `ensures: result == x + 999999` on a constant input is fully
// constant-provable after inlining; it must still abort at --opt-level 2.
// Expected: abort (exit 1 + `contract violated`) at --opt-level 2.
// Pure XIOM. Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m311_o2_contract_ens

use xiom.io;

fn short_value(x: Int) -> Int
  ensures: result == x + 999999
{
  return x;
}

fn main() -> Int {
  io.println("before");
  let v = short_value(7);
  io.println("UNREACHABLE v=" + to_string(v));
  return 0;
}
