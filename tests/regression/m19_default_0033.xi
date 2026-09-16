// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0033

interface Divisible {
  fn div2(&self) -> Result[Int, Str] {
    var v = value();
    if v % 2 == 0 { return Ok(v / 2); }
    return Err("odd");
  }
  fn value(&self) -> Int;
}

type Even = { num: Int; }

fn Even.value(&self) -> Int { return num; }

fn main() -> Int {
  var e: Even = Even{ num: 10 };
  var r = e.div2();
  match r {
    Ok(n) => if n == 5 { return 0; },
    Err(_) => return 2
  }
  return 1;
}
