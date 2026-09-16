// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_destructure_008
pub fn run() -> Int {
    var t = (Some(42), Ok(100), "test");
    match t.0 {
      Some(v) => if v == 42 { return 0; },
      None => return 1,
    }
  }
use m21_destructure_008.run;
fn main() -> Int { return run(); }
