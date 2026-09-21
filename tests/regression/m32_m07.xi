// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M07: pub const -- public constant declared in module
module config {
  pub const MAX_SIZE: Int = 256;
}
fn main() -> Int {
  if config.MAX_SIZE == 256 { return 0; }
  return 1;
}
