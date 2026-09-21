// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0109

interface Bounds {
  fn pair(&self) -> Pair { return Pair{ first: min(), second: max() }; }
  fn min(&self) -> Int;
  fn max(&self) -> Int;
}

type Pair = { first: Int; second: Int; }

type Range = { lo: Int; hi: Int; }

fn Range.min(&self) -> Int { return lo; }
fn Range.max(&self) -> Int { return hi; }

fn main() -> Int {
  var r: Range = Range{ lo: 3, hi: 42 };
  var p = r.pair();
  if p.first != 3 { return 1; }
  if p.second != 42 { return 2; }
  return 0;
}
