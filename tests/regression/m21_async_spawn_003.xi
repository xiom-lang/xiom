// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_async_spawn_003
async fn compute(x: Int) -> Int {
    return x * 2;
  }

  pub fn run() -> Int {
    return 0;
  }
use m21_async_spawn_003.run;
fn main() -> Int { return run(); }
