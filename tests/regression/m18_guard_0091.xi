// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0091

fn is_valid(n: Int) -> Bool {
  if n <= 1 { return n == 1; }
  if n % 2 == 0 { return is_valid(n / 2); }
  return is_valid(n - 1);
}

fn main() -> Int {
  var v: Int = 8;
  match v {
    v if is_valid(v) => { return 0; }
    _ => { return 1; }
  }
}
