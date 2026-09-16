// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0053

enum Status {
  Ok
  Error(code: Int)
}

fn main() -> Int {
  var s: Status = Error(500);
  match s {
    Error(c) if c == 404 => { return 1; }
    Error(c) => { return 0; }
    _ => { return 2; }
  }
}
