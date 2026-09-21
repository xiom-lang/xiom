// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Result Int8 payload (Ok)
fn main() -> Int {
  var r: Result[Int8, Bool] = Ok(-128 as Int8);
  if r.is_ok() { return 0; }
  return 1;
}
