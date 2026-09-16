// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_017
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(1);
    if r.is_ok() { return 0; }
    elif r.is_err() { return 1; }
    else { return 1; }
  }
use m21_if_chain_017.run;
fn main() -> Int { return run(); }
