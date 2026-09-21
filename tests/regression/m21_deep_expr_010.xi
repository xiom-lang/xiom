// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_010
pub fn run() -> Int {
    var a = 10;
    var b = 20;
    var c = 30;
    var d = 40;
    var e = 50;
    var f = 60;
    if a < b {
      if b < c {
        if c < d {
          if d < e {
            if e < f {
              if a + b + c + d + e + f == 210 { return 0; }
            }
          }
        }
      }
    }
    return 1;
  }
use m21_deep_expr_010.run;
fn main() -> Int { return run(); }
