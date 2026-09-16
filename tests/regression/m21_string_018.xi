// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_018
pub fn run() -> Int {
    var r: Result[Str, Int] = Ok("success");
    match r {
      Ok(msg) => if msg == "success" { return 0; },
      Err(_) => return 1,
    }
  }
use m21_string_018.run;
fn main() -> Int { return run(); }
