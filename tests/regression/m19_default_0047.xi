// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0047

interface Ranked {
  fn rank(&self) -> Int {
    var v = value();
    if v > 100 { return 3; }
    elif v > 50 { return 2; }
    elif v > 0 { return 1; }
    else { return 0; }
  }
  fn value(&self) -> Int;
}

type Score = { points: Int; }

fn Score.value(&self) -> Int { return points; }

fn main() -> Int {
  var hi: Score = Score{ points: 200 };
  var mid: Score = Score{ points: 75 };
  var lo: Score = Score{ points: 25 };
  var zero: Score = Score{ points: -5 };
  if hi.rank() != 3 { return 1; }
  if mid.rank() != 2 { return 2; }
  if lo.rank() != 1 { return 3; }
  if zero.rank() != 0 { return 4; }
  return 0;
}
