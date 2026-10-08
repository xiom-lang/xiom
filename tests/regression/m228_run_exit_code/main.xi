// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m228 (B-08, bindings-lane tooling): `xiom --run` used to exit 0 no matter
// what the program returned; suites whose main returned 5 saw success.
module m228_run_exit_code

fn main() -> Int {
  return 5;
}
