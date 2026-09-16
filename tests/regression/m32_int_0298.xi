// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Option UInt16 with high value
fn main() -> Int {
  var opt: Option[UInt16] = Some(60000);
  var v: UInt16 = opt.unwrap();
  if v == 60000 as UInt16 { return 0; }
  return 1;
}
