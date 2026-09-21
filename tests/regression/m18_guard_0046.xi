// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0046

enum Value {
  None
  Int32(n: Int)
}

fn main() -> Int {
  var v: Value = Int32(42);
  match v {
    Int32(n) if n > 10 => { return 0; }
    _ => { return 1; }
  }
}
