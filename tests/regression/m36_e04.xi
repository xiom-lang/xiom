// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E04: Empty enum -- minimal enum with no payload variants (unit variants only)
enum VoidE { None }
fn main() -> Int {
  var v = VoidE.None;
  var ok = 0;
  match v {
    None => { ok = 1; }
  }
  if ok != 1 { return 1; }
  return 0;
}
