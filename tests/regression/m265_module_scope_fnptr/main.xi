// m265 (REL-20261011-0317 pulse-7): MODULE-SCOPE fn-pointer storage.
// `var handler: fn(Int) -> Int = f0;` is callable and REASSIGNABLE
// (`handler = f1;`). Pre-fix the bare call resolved as a missing FUNCTION
// ("unresolved function symbol 'handler'") -- the test-harness wishlist
// shape (swap a runner fn per test). Exit 0 on success.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m265_module_fnptr

fn f0(x: Int) -> Int {
  return 1;
}

fn f1(x: Int) -> Int {
  return 2;
}

var handler: fn(Int) -> Int = f0;

fn main() -> Int {
  if handler(0) != 1 {
    return 1;
  }
  handler = f1;
  if handler(0) != 2 {
    return 2;
  }
  handler = f0;
  if handler(0) != 1 {
    return 3;
  }
  return 0;
}
