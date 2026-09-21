// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_async_spawn_008
async fn retry(f: fn() -> Result[Int, Int]) -> Result[Int, Int] {
    return Ok(0);
  }

  pub fn run() -> Int {
    return 0;
  }
use m21_async_spawn_008.run;
fn main() -> Int { return run(); }
