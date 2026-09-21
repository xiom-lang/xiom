// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E05: Single-variant enum
enum Solo { Only(val: Int) }
fn main() -> Int {
  var s = Solo.Only(7);
  match s {
    Only(v) => { if v != 7 { return 1; } }
  }
  return 0;
}
