// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m250 (stdlib typechanging family): cross-type generic callbacks
// (fn(T) -> U with U = Str) silently miscompiled:
//  - a generic call's local binding recorded the DECLARED return
//    ("Vec[U]"/"Option[U]"), so `a[0]`/match arms read the element as an
//    i64 truncated to a byte;
//  - the `Vec[U]` RETURN arm was missing entirely (name degraded to "Vec");
//  - method calls misaligned fd.params (self at [0]) with args, so the
//    fn-typed param never inferred U;
//  - closure-literal callbacks had no signature inference.
// Covers fn-ref, convert-wrapping fn-ref, closure literal, U=Int
// baseline, and the core Option.map(to_s) receiver shape.
module m250_typechanging_callbacks

use xiom.string;
use xiom.convert;
use xiom.core;

fn mapv[T, U](v: &Vec[T], f: fn(T) -> U) -> Vec[U] {
  var out = Vec[U].new();
  var i = 0;
  while i < v.len() {
    out.push(f(v[i]));
    i = i + 1;
  }
  return out;
}

fn lit_s(x: Int) -> Str {
  if x == 1 { return "1"; }
  return "2";
}

fn conv_s(x: Int) -> Str { return convert.int_to_string(x); }

fn dbl(x: Int) -> Int { return x * 2; }

fn apply[T, U](x: T, f: fn(T) -> U) -> U {
  return f(x);
}

fn main() -> Int {
  var v: Vec[Int] = Vec[Int].new();
  v.push(1); v.push(2);

  // fn-ref with constant-string body
  let a = mapv(&v, lit_s);
  if a.len() != 2 { return 10; }
  if str_compare(a[0], "1") != 0 { return 11; }
  if str_compare(a[1], "2") != 0 { return 12; }

  // fn-ref wrapping convert
  let b = mapv(&v, conv_s);
  if b.len() != 2 { return 20; }
  if str_compare(b[0], "1") != 0 { return 21; }
  if str_compare(b[1], "2") != 0 { return 22; }

  // closure literal callback
  let c = mapv(&v, fn(x: Int) -> Str { if x == 1 { return "1"; } return "2"; });
  if c.len() != 2 { return 30; }
  if str_compare(c[0], "1") != 0 { return 31; }

  // U = Int baseline
  let d = mapv(&v, dbl);
  if d.len() != 2 { return 40; }
  if d[0] != 2 { return 41; }

  // callback through a generic without the Vec (return ABI only)
  let e = apply(1, lit_s);
  if str_compare(e, "1") != 0 { return 50; }

  // core Option.map with a type-changing callback, matched by payload
  let o = Some(7);
  match o.map(conv_s) {
    Some(s) => {
      if str_compare(s, "7") != 0 { return 60; }
    };
    None => { return 61; }
  }

  return 0;
}
