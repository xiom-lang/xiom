// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_019
fn get_val() -> Int { return 42; }

  pub fn run() -> Int {
    if get_val() == 42 { return 0; }
    elif get_val() == 0 { return 1; }
    else { return 1; }
  }
use m21_if_chain_019.run;
fn main() -> Int { return run(); }
