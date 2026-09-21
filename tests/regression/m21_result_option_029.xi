// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_029
type Response = { status: Result[Int, Int]; body: Int; }

  pub fn run() -> Int {
    var resp: Response = { status: Err(500); body: 0; };
    match resp.status {
      Ok(_) => return 1,
      Err(code) => if code == 500 { return 0; },
    }
  }
use m21_result_option_029.run;
fn main() -> Int { return run(); }
