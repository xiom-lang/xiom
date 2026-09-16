// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_014
pub fn run() -> Int {
    var b: Bool = true;
    match b {
      true => return 0,
      false => return 1,
    }
  }
use m21_match_edge_014.run;
fn main() -> Int { return run(); }
