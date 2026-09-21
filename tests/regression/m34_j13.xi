// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J13: Pub fn calling private fn -- public wrapper around private implementation
module range {
  fn clamp_inner(v: Int, lo: Int, hi: Int) -> Int {
    if v < lo { return lo; }
    if v > hi { return hi; }
    return v;
  }
  fn scale_inner(v: Int, factor: Int) -> Int { return v * factor; }
  pub fn clamp(v: Int, lo: Int, hi: Int) -> Int { return clamp_inner(v, lo, hi); }
  pub fn normalize(v: Int, lo: Int, hi: Int) -> Int {
    var c = clamp_inner(v, lo, hi);
    return scale_inner(c - lo, 100) / (hi - lo);
  }
}
use range.clamp;
fn main() -> Int {
  var c1 = clamp(-5, 0, 10);
  var c2 = clamp(15, 0, 10);
  var c3 = clamp(5, 0, 10);
  if c1 == 0 && c2 == 10 && c3 == 5 { return 0; }
  return 1;
}
