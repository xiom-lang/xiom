// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0082

type Data = { x: Int; }

fn main() -> Int {
  var x: Int = 100;
  var d: Data = Data{ x: 42 };
  match d {
    d if d.x > 0 => { return 0; }
    _ => { return 1; }
  }
}
