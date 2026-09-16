// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_async_spawn_005
async fn fetch_data(url: Str) -> Str {
    return "data";
  }

  pub fn run() -> Int {
    return 0;
  }
use m21_async_spawn_005.run;
fn main() -> Int { return run(); }
