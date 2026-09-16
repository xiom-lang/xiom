// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0109

fn main() -> Int {
  var s: Str = "hello";
  match s {
    v if v == "hello" => { return 0; }
    _ => { return 1; }
  }
}
