// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_003
pub fn run() -> Int {
    var a = "hello";
    var b = "world";
    var c = a.concat(b);
    if c == "helloworld" { return 0; }
    return 1;
  }
use m21_string_003.run;
fn main() -> Int { return run(); }
