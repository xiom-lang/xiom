// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M03: Private fn -- private helper called by pub fn
module secrets {
  fn double(x: Int) -> Int { return x * 2; }
  pub fn quadruple(x: Int) -> Int { return double(double(x)); }
}
use secrets.quadruple;
fn main() -> Int {
  if quadruple(5) == 20 { return 0; }
  return 1;
}
