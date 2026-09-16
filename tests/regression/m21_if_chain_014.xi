// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_014
pub fn run() -> Int {
    var x = 10;
    var y = 20;
    if x > 0 {
      if y > 10 {
        if x + y > 25 {
          if y - x == 10 {
            return 0;
          }
        }
      }
    }
    return 1;
  }
use m21_if_chain_014.run;
fn main() -> Int { return run(); }
