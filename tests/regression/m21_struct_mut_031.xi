// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_031
type Counter = { val: Int; }
fn main() -> Int {
  var c: Counter = Counter{ val: 0; };
  c.val = c.val + 1;
  c.val = c.val + 1;
  if c.val == 2 { return 0; }
  return 1;
}
