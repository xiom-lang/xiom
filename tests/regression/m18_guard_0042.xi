// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0042

type Point = { x: Int; y: Int; }

fn main() -> Int {
  var p: Point = { x: 5; y: 10; };
  match p {
    p if p.x > 10 => { return 1; }
    _ => { return 0; }
  }
}
