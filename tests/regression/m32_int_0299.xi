// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Option UInt32 with value above Int32 max
fn main() -> Int {
  var opt: Option[UInt32] = Some(3000000000);
  var v: UInt32 = opt.unwrap();
  if v == 3000000000 as UInt32 { return 0; }
  return 1;
}
