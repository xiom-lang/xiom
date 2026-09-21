// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_contract_001
fn divide(a: Int, b: Int) -> Result[Int, Str]
    requires: b != 0
  {
    if b == 0 { return Err("div by zero"); }
    return Ok(a / b);
  }

  pub fn run() -> Int {
    var r = divide(10, 2);
    match r {
      Ok(v) => if v == 5 { return 0; },
      Err(_) => return 1,
    }
  }
use m21_contract_001.run;
fn main() -> Int { return run(); }
