// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_008
pub fn run() -> Int {
    var x = 0;
    if true {
      if true {
        if true {
          if true {
            if true {
              if true {
                if true {
                  if true {
                    x = 1;
                  }
                }
              }
            }
          }
        }
      }
    }
    if x == 1 { return 0; }
    return 1;
  }
use m21_deep_expr_008.run;
fn main() -> Int { return run(); }
