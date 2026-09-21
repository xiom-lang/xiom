// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt8 to Bool with nonzero
fn main() -> Int {
  var u: UInt8 = 5 as UInt8;
  var as_bool: Bool = u != 0 as UInt8;
  if as_bool == true { return 0; }
  return 1;
}
