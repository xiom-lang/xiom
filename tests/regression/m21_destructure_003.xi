// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_destructure_003
type Triple = { first: Int; second: Bool; third: Str; }

  pub fn run() -> Int {
    var t: Triple = { first: 1; second: true; third: "ok"; };
    if t.first == 1 && t.second && t.third == "ok" { return 0; }
    return 1;
  }
use m21_destructure_003.run;
fn main() -> Int { return run(); }
