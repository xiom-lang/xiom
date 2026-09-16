// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T18: Module importing with use
module m {
  pub fn answer() -> Int { return 42; }
}
use m.answer;
fn main() -> Int {
  if answer() != 42 { return 1; }
  return 0;
}

