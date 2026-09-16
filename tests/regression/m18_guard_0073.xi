// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0073

fn main() -> Int {
  var flag: Bool = true;
  match flag {
    v if v => { return 0; }
    _ => { return 1; }
  }
}
