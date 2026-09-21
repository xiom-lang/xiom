// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module b9main
use b9mod;
use xiom.io;

fn main() -> Int {
  var f = b9mod.make_frac(3, 4);
  var s = b9mod.frac_sum(f);
  io.println("sum=" + s.to_str());
  if s != 7 { return 1; }
  return 0;
}
