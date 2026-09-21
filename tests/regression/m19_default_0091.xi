// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0091

interface SafeDiv {
  fn safe_div(&self) -> Result[Int, Str] {
    var v = value();
    var d = divisor();
    if d == 0 { return Err("div zero"); }
    return Ok(v / d);
  }
  fn value(&self) -> Int;
  fn divisor(&self) -> Int;
}

type Pair = { v: Int; d: Int; }

fn Pair.value(&self) -> Int { return v; }

fn Pair.divisor(&self) -> Int { return d; }

fn main() -> Int {
  var p: Pair = Pair{ v: 10, d: 2 };
  var r = p.safe_div();
  match r {
    Ok(n) => if n == 5 { return 0; },
    Err(_) => return 2
  }
  return 1;
}
