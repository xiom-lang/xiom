// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module catfix_main
use vecmod;
use vecmod.Wrap;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  var s = vecmod.vec_sum(&v);
  if s != 6 { return 1; }
  var w = Wrap{ a: 42; b: 7; };
  if vecmod.wrap_a(&w) != 42 { return 2; }
  return 0;
}
