// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-K20: Full closure composition -- fn ptr + pipe + block + chain
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }
fn dbl_named(x: Int) -> Int { return x * 2; }
fn main() -> Int {
  var inc = |x| x + 1;
  var dbl = fn(x: Int) -> Int { return x * 2; };
  if inc(5) != 6 { return 1; }
  if dbl(10) != 20 { return 2; }
  if apply(dbl_named, 21) != 42 { return 3; }
  var chain = inc(dbl(10));
  if chain != 21 { return 4; }
  return 0;
}
