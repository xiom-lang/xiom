// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_040
type Entry = { key: Int; val: Str; }

  fn get_val(e: Entry) -> Int {
    return e.key;
  }

  pub fn run() -> Int {
    var e: Entry = { key: 7; val: "test"; };
    var k = get_val(e);
    if k == 7 { return 0; }
    return 1;
  }

use m21_struct_mut_040.run;
fn main() -> Int { return run(); }
