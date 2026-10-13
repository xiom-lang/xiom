// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m299 lock (REL-20261010-1549-packages-12): destructuring a REFERENCE to a
// tuple element (`let (k, v) = &vec[i]`) must bind VALUE COPIES. The m209
// ref binding leaked component addresses into value uses (prints showed
// decimal addresses; the Str concat path deref'd through a byte load ->
// 0xC0000005); the workaround was bind-then-destructure. Value semantics are
// now identical to that workaround and the m209 `k == &key` content compare
// keeps working. Each failing arm adds a distinct bit so the exit code names
// the broken shape.
module m299_tuple_ref_destructure;

use xiom.io;
use xiom.convert;

fn main() -> Int {
  var fails: Int = 0;

  // A: Vec[(Int, Int)] -- ref-destructure copies the Int components; the
  // string built from them gates on the actual numbers (an address string
  // fails the compare).
  var a = Vec[(Int, Int)].new();
  a.push((7, 9));
  let (ak, av) = &a[0];
  if ak != 7 { fails = fails + 1; }
  if av != 9 { fails = fails + 2; }
  let astr = convert.int_to_string(ak) + "|" + convert.int_to_string(av);
  if astr != "7|9" { fails = fails + 4; }

  // B: Vec[(Str, Str)] -- the grpc metadata shape; concat of the copied
  // strings must not crash (byte-load deref pre-fix) and must match.
  var c = Vec[(Str, Str)].new();
  c.push(("grpc-timeout", "5s"));
  let (ck, cv) = &c[0];
  if ck != "grpc-timeout" { fails = fails + 8; }
  if cv != "5s" { fails = fails + 16; }
  let cs = ck + "|" + cv;
  if cs != "grpc-timeout|5s" { fails = fails + 32; }

  // C: the m209 compare shape must stay working (`k == &key` content
  // compare with a loop-variable index).
  var i = 0;
  var found = false;
  while i < c.len() {
    let (k2, v2) = &c[i];
    if k2 == &ck {
      if v2 == &cv { found = true; }
    };
    i = i + 1;
  };
  if !found { fails = fails + 64; }

  // D: plain by-value element destructure (control).
  let (dk, dv) = c[0];
  if dk != "grpc-timeout" { fails = fails + 128; }
  if dv != "5s" { fails = fails + 256; }

  // E: bind-then-destructure workaround (control).
  let e = a[0];
  let (bk, bv) = e;
  if bk != 7 { fails = fails + 512; }
  if bv != 9 { fails = fails + 1024; }

  io.println("m299_tuple_ref_destructure fails=" + convert.int_to_string(fails));
  return fails;
}
