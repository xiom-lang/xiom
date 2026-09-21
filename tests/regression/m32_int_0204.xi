// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Option UInt8 payload
fn main() -> Int {
  var opt: Option[UInt8] = Some(255);
  if opt.unwrap() == 255 as UInt8 { return 0; }
  return 1;
}
