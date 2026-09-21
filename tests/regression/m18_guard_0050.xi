// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0050

enum Msg {
  Start
  Data(v: Int)
  Stop
}

fn main() -> Int {
  var m: Msg = Data(0);
  match m {
    Data(v) if v > 10 => { return 1; }
    _ => { return 0; }
  }
}
