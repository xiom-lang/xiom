// m259 (REL-20261010-1900): a body whose resolution already failed (T001
// undefined variable) and a bare-variant match over an unresolved enum must
// NOT get W004 reachability cascades -- rustc gates `check_match` behind
// `Result<(), ErrorGuaranteed>`; we gate on body error taint + an
// error-typed scrutinee.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module p_w004_no_cascade;

use xiom.engine.core.events;

fn tick(e: Event) -> Int {
  var k: Int = 0;
  let z = MissingName;
  match e {
    Startup => k = 1;
    Shutdown => k = 2;
    FixedTick(t) => k = 3;
    Frame(d) => k = 4;
    Resize(w, h) => k = 5;
    Key(c, p) => k = 6;
  }
  return k + z;
}

fn main() -> Int {
  return 0;
}
