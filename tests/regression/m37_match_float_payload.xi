// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_match_float_payload
// BUG 22 #2/#4 regression: match-bound Option payload vars.
// Some(5.0) stores the double BITS in the i64 payload slot; the arm
// extraction must bitcast back so `-d`/arithmetic on the bound var work.
// Also: `-d` on a match-bound var previously failed the checker with
// "cannot negate type _" (wildcard convention).

fn main() -> Int {
  var o = Some(5.0);
  var neg = 0.0;
  match o {
    Some(d) => neg = -d,
    _ => neg = 0.0,
  }
  if neg != -5.0 { return 1; }
  // Ok payload through match
  var r = Ok(2.5);
  var got = 0.0;
  match r {
    Ok(v) => got = v,
    Err(e) => got = -1.0,
  }
  if got != 2.5 { return 2; }
  // Int payload unaffected
  var oi = Some(7);
  var iv = 0;
  match oi {
    Some(n) => iv = n,
    _ => iv = -1,
  }
  if iv != 7 { return 3; }
  return 0;
}
