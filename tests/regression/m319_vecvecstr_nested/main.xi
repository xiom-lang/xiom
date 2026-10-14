// m319 (REL-20261013-2238-engine, vecvecstr): a Vec[Vec[Str]] element READ
// returned the string HANDLE as an untyped i64 through several consumer
// paths -- `inner[i]` (inner bound from `outer[0]`), `let s = outer[0][1]`
// concatenated, a nested element passed to a Str param, and a read off a
// function-returned nested Vec -- because the index site's element-type
// resolution only knew Ident/Field containers and the binding did not record
// the element's XIOM type. Symptoms: decimal pointer digits / mojibake / an
// empty string (the engine's "nested element reads return garbage").
// Pre-fix this fixture returns nonzero (fails 4); the direct nested concat
// (shape A) is the control.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m319_vecvecstr_nested.main;

use xiom.io;
use xiom.convert;

fn mk() -> Vec[Vec[Str]] {
  var o = Vec[Vec[Str]].new();
  var v = Vec[Str].new();
  v.push("zz");
  o.push(v);
  return o;
}

fn echo(t: Str) -> Str { return t; }

fn main() -> Int {
  var fails: Int = 0;

  var outer = Vec[Vec[Str]].new();
  var a = Vec[Str].new();
  a.push("aa");
  a.push("bb");
  outer.push(a);
  var b = Vec[Str].new();
  b.push("cc");
  outer.push(b);

  // A: direct nested index in a concat (control -- green pre-fix too).
  let s1 = outer[0][0] + "!";
  if s1 != "aa!" { fails = fails + 1; }

  // B: bound inner vec, element concat (was decimal pointer digits).
  var inner = outer[0];
  if inner.len() != 2 { fails = fails + 1; }
  let s2 = inner[1] + "!";
  if s2 != "bb!" { fails = fails + 1; }

  // C: bound Str element from a nested read (was pointer digits).
  let s = outer[0][1];
  let s3 = s + "!";
  if s3 != "bb!" { fails = fails + 1; }

  // D: nested element through a Str param (was mojibake/empty).
  if echo(outer[0][0]) != "aa" { fails = fails + 1; }

  // E: nested read off a function-returned Vec[Vec[Str]] (was empty).
  if mk()[0][0] != "zz" { fails = fails + 1; }

  // F: second outer row (index 1) keeps the same resolution.
  if outer[1][0] != "cc" { fails = fails + 1; }

  io.println("m319_vecvecstr_nested: fails=" + int_to_string(fails)
      + " s1=" + s1 + " s2=" + s2 + " s3=" + s3);
  return fails;
}
