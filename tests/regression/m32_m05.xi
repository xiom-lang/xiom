// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M05: use type alias -- import pub type from module
module types {
  pub type Score = Int;
  pub fn create_score(n: Int) -> Score { return n; }
}
use types.Score;
use types.create_score;
fn main() -> Int {
  var s: Score = create_score(99);
  if s == 99 { return 0; }
  return 1;
}
