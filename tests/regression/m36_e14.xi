// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E14: Underscore-prefixed names
fn _helper(x: Int) -> Int { return x * 2; }
type _Internal = { _val: Int; }
fn main() -> Int {
  var _x = 5;
  var _r = _helper(_x);
  var _s = _Internal{ _val: _r };
  if _s._val != 10 { return 1; }
  return 0;
}
