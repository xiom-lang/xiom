// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Result UInt16 Err type with unwrap_or
fn main() -> Int {
  var r: Result[Bool, UInt16] = Err(50000);
  if r.is_err() { return 0; }
  return 1;
}
