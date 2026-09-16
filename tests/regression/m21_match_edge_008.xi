// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_008
pub fn run() -> Int {
    var r: Result[Int, Int] = Err(404);
    match r {
      Ok(v) if v > 0 => return 1,
      Ok(_) => return 1,
      Err(e) if e == 404 => return 0,
      Err(e) if e == 500 => return 1,
      Err(_) => return 1,
    }
  }
use m21_match_edge_008.run;
fn main() -> Int { return run(); }
