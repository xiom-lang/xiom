// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_008
pub fn run() -> Int {
    var cmd = "start";
    if cmd == "start" { return 0; }
    elif cmd == "stop" { return 1; }
    else { return 2; }
  }
use m21_string_008.run;
fn main() -> Int { return run(); }
