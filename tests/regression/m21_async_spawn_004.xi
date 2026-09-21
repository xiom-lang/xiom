// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_async_spawn_004
pub fn run() -> Int {
    spawn {
      var sum = 0;
      var i = 0;
      while i < 10 {
        sum = sum + i;
        i = i + 1;
      }
    }
    return 0;
  }
use m21_async_spawn_004.run;
fn main() -> Int { return run(); }
