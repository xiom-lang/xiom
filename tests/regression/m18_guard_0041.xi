// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0041

type Point = { x: Int; y: Int; }

fn main() -> Int {
  var p: Point = { x: 5; y: 10; };
  match p {
    p if p.x > 0 => { return 0; }
    _ => { return 1; }
  }
}
