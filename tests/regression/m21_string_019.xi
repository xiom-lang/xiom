// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_019
pub fn run() -> Int {
    var v: Vec[Str] = ["a", "b", "c"];
    var i = 0;
    var found = false;
    while i < v.len() {
      if v[i] == "b" { found = true; }
      i = i + 1;
    }
    if found { return 0; }
    return 1;
  }
use m21_string_019.run;
fn main() -> Int { return run(); }
