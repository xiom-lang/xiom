// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0079

enum Dir { N, S, E, W }

fn main() -> Int {
  var d: Dir = N;
  match d {
    N | S | E if true => { return 0; }
    _ => { return 1; }
  }
}
