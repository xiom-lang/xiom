// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_016
pub fn run() -> Int {
    var opt: Option[Int] = Some(42);
    if opt.is_some() { return 0; }
    elif opt.is_none() { return 1; }
    else { return 1; }
  }
use m21_if_chain_016.run;
fn main() -> Int { return run(); }
