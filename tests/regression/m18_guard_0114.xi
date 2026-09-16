// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0114

fn main() -> Int {
  var i: Int = 0;
  var result: Int = 0;
  while i < 5 {
    match i {
      v if v % 2 == 0 => { result = result + v; }
      _ => { }
    }
    i = i + 1;
  }
  if result == 6 { return 0; }
  return 1;
}
