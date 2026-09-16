// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0089

type Box = { val: Int; }

fn Box.double(&self) -> Int {
  return val * 2;
}

fn main() -> Int {
  var b: Box = Box{ val: 51 };
  match b {
    v if v.double() > 100 => { return 0; }
    _ => { return 1; }
  }
}
