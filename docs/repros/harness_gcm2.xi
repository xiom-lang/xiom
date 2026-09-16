// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_gcm2;

fn main() -> Int {
  var enc = make_pair();
  match enc {
    Ok(pair) => {
      // THE SMOKE SHAPE: &pair.0 / &pair.1 (tuple field REFERENCE)
      var r = consume_pair(&pair.0, &pair.1);
      if r != 0 { return r; }
      return 0;
    }
    Err(_) => { return 9; }
  }
}
