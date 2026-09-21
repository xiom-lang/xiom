// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 43 regression: Float64 payload in generic Result/Option read via
// sitofp (should bitcast). Direct-CALL scrutinee (`match core.to_float_from_str(s)`)
// must bind the payload as a real double, not the raw bit pattern (~4.6e18).
module m37_bug43_result_f64_payload
use xiom.core;

fn main() -> Int {
  match core.to_float_from_str("3.14") {
    Ok(f) => {
      if f < 3.13 || f > 3.15 { return 11; }
    },
    Err(_) => { return 12; },
  };
  var r: Result[Float64, Str] = core.to_float_from_str("2.5");
  match r {
    Ok(g) => {
      if g != 2.5 { return 21; }
    },
    Err(_) => { return 22; },
  };
  // Err payload must stay a real Str (inttoptr, not a byte value).
  match core.to_float_from_str("not-a-number") {
    Ok(_) => { return 31; },
    Err(e) => {
      if e.len() == 0 { return 32; }
    },
  };
  return 0;
}
