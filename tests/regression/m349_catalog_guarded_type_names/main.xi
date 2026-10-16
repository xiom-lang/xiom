// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m349 lock (REL-20261016-2004-packages / XARROW-ARRAYNAME): a source-dir
// catalog module (the installed-package model) declaring `pub type Array`
// used to be silently dropped by the injection PRIMITIVES filter -- codegen
// had no layout for `Array`, fell back to i64 ("unknown type 'Array' --
// defaulting to i64") and every field read returned zero (arrow's C struct
// named `Array`; pre-fix rc 31). Post-fix the declaration injects like any
// struct and the exact values must survive every shape below.
module m349_catalog_guarded_type_names.main;

use xiom.io;
use xiom.m349x.def;

pub fn main() -> Int {
  var fail = 0;
  let a = mk_arr(7);
  io.println("arr n=" + a.n.to_str() + " m=" + a.m.to_str());
  if a.n != 7 { fail = fail + 1; }
  if a.m != 14 { fail = fail + 2; }
  let s = arr_sum(&a);
  io.println("arr sum=" + s.to_str());
  if s != 21 { fail = fail + 4; }
  let b = Array { n: 1, m: 2 };
  io.println("lit n=" + b.n.to_str() + " m=" + b.m.to_str());
  if b.n != 1 { fail = fail + 8; }
  if b.m != 2 { fail = fail + 16; }
  let h = mk_holder(5);
  let hs = holder_sum(&h);
  io.println("holder sum=" + hs.to_str());
  if hs != 26 { fail = fail + 32; }
  let sp = a.span();
  if sp != 21 { fail = fail + 64; }
  let vs = mk_arr_vec(3);
  let vsum = vs[0].n + vs[0].m;
  io.println("vec elem sum=" + vsum.to_str());
  if vsum != 9 { fail = fail + 128; }
  io.println("m349 fail_bits=" + fail.to_str());
  return fail;
}
