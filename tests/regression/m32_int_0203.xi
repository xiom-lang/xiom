// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Option Int16 payload
fn main() -> Int {
  var opt: Option[Int16] = Some(32767);
  if opt.is_some() { return 0; }
  return 1;
}
