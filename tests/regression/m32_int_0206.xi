// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Result UInt16 payload
fn main() -> Int {
  var r: Result[UInt16, Bool] = Ok(65535);
  if r.unwrap() == 65535 as UInt16 { return 0; }
  return 1;
}
