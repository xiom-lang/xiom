// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m237 (wave-96 p_array_zip_no_truncate): `array_zip` must truncate to the
// SHORTER array. Pre-fix the const-generic inference bound M to N's size
// (both mono'd as array_zip_3_3), so `if M < count` was dead and the loop
// read b[M] out of bounds for M < N.
module m237_array_zip_truncate

use xiom.array.fixed as afix;

fn main() -> Int {
  // M < N: two pairs, content checked.
  let a3 = [1, 2, 3];
  let b2 = [7, 8];
  let z = afix.array_zip(&a3, &b2);
  if z.len() != 2 { return 1; }
  if z[0].0 != 1 { return 2; }
  if z[0].1 != 7 { return 3; }
  if z[1].0 != 2 { return 4; }
  if z[1].1 != 8 { return 5; }

  // M == 0: zero pairs (no OOB probe).
  let b0 = [];
  let z0 = afix.array_zip(&a3, &b0);
  if z0.len() != 0 { return 6; }

  // N <= M control: still N pairs.
  let c2 = [4, 5];
  let d3 = [9, 10, 11];
  let zn = afix.array_zip(&c2, &d3);
  if zn.len() != 2 { return 7; }
  if zn[1].0 != 5 { return 8; }
  if zn[1].1 != 10 { return 9; }

  // N == M: all pairs.
  let e2 = [6, 7];
  let zm = afix.array_zip(&c2, &e2);
  if zm.len() != 2 { return 10; }
  if zm[1].1 != 7 { return 11; }

  // Acceptance form: array LITERAL arguments (`&[1,2,3]`, not a local).
  let zl = afix.array_zip(&[1, 2, 3], &[7, 8]);
  if zl.len() != 2 { return 12; }
  if zl[0].0 != 1 { return 13; }
  if zl[1].1 != 8 { return 14; }

  return 0;
}
