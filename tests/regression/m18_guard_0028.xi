// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0028

type Data = { value: Int; }

fn is_positive(n: Int) -> Bool { return n > 0; }

fn main() -> Int {
  var d: Data = { value: 7; };
  match d {
    v if is_positive(v.value) => { return 0; }
    _ => { return 1; }
  }
}
