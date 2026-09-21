// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_internal_destructure;

fn main() -> Int {
  var r = roundtrip();
  match r {
    Ok(pair) => {
      var a = pair.0;
      var b = pair.1;
      if a.len() != 2 { return 1; }
      if a[0] != 1 { return 2; }
      if a[1] != 2 { return 3; }
      if b.len() != 14 { return 4; }
      if b[13] != 13 { return 5; }
      return 0;
    }
    Err(_) => { return 9; }
  }
}
