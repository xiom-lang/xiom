// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m344 negative lock (REL-20261011-1046-engine): the copy-type exemption is
// kept for genuine scalars. A scalar (Int) FIELD projection and a scalar
// CALL-INIT local may be reused freely -- the moved-value coverage must not
// turn their reuse into diagnostics (default or --strict).
module m344_scalar_field_clean;

type S = { n: Int; }

fn count() -> Int { return 7; }
fn take(n: Int) -> Int { return n; }

fn main() -> Int {
  // (1) scalar field projection reuse: Copy -- stays clean.
  var a = S { n: 1 };
  let r1 = take(a.n);
  let r2 = take(a.n);
  if r1 != 1 { return 1; }
  if r2 != 1 { return 2; }

  // (2) scalar call-init local reuse: Copy -- stays clean.
  var n = count();
  let r3 = take(n);
  let r4 = take(n);
  if r3 != 7 { return 3; }
  if r4 != 7 { return 4; }
  return 0;
}
