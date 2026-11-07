// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// REL-20261010-1553 (v0.65.6 Str follow-up): an INDIRECT comparator call
// (`fn(&Str, &Str) -> Bool` held in a fn-typed parameter) fed the VALUE
// produced by `Vec.get(...).unwrap()`. The materializer correctly turned the
// `Str` value (`i8*`) into an `i64` slot address, but the call site kept the
// PRE-materialization type for the emitted fn-pointer signature:
//
//   %tmp115 = inttoptr i64 %tmp114 to i64 (i64, i8*, i8*)*
//   %tmp116 = call i64 %tmp115(i64 %tmp112, i8* %tmp65, i8* %tmp111)
//
// while `%tmp65`/`%tmp111` were `ptrtoint ... to i64` addresses -> clang:
// "'%tmp65' defined with type 'i64' but expected 'ptr'". The Int form never
// hit this because its value type was already `i64`; the Str form did.
// Post-fix the fn-pointer signature is the uniform `i64 (i64, i64, i64)*`.
module m308_ref_str_cmp_value_args.main

use xiom.io;

fn less_str(a: &Str, b: &Str) -> Bool { return a < b; }
fn less_int(a: &Int, b: &Int) -> Bool { return a < b; }

// Generic, by-value Vec: the L6-34 `items.get(i).unwrap()` value-arg shape.
fn find_min_vec[T](items: Vec[T], less: fn(&T, &T) -> Bool) -> Int
{
  var best: Int = 0;
  var i: Int = 1;
  while i < items.len() {
    if less(items.get(i).unwrap(), items.get(best).unwrap()) { best = i; }
    i = i + 1;
  }
  return best;
}

// Non-generic, by-ref Vec: dynamic refs through the same value-arg path.
fn find_min_ref(items: &Vec[Str], less: fn(&Str, &Str) -> Bool) -> Int
{
  var best: Int = 0;
  var i: Int = 1;
  while i < items.len() {
    if less(items.get(i).unwrap(), items.get(best).unwrap()) { best = i; }
    i = i + 1;
  }
  return best;
}

fn main() -> Int
{
  // 1. Generic + named `fn(&Str, &Str)` + Vec.get().unwrap() values. A
  //    by-value Vec param MOVES, so keep a dedicated vec here.
  var vg: Vec[Str] = Vec[Str].new();
  vg.push("pear");
  vg.push("cherry");
  vg.push("apple");
  let i1 = find_min_vec[Str](vg, less_str);
  if i1 != 2 { io.println("generic=" + i1.to_str()); return 1; }

  var vr: Vec[Str] = Vec[Str].new();
  vr.push("pear");
  vr.push("cherry");
  vr.push("apple");

  // 2. Non-generic + by-ref Vec + named comparator.
  let i2 = find_min_ref(&vr, less_str);
  if i2 != 2 { io.println("byref=" + i2.to_str()); return 2; }

  // 3. Closure literal comparator (unannotated `|a, b|`) through the same
  //    value-arg path; the declared `&Str` params come from the hint.
  let i3 = find_min_ref(&vr, |a, b| a < b);
  if i3 != 2 { io.println("closure=" + i3.to_str()); return 3; }

  // 4. Int control: the value-arg path must stay clean for scalars.
  var vi: Vec[Int] = Vec[Int].new();
  vi.push(5);
  vi.push(12);
  vi.push(3);
  let i4 = find_min_vec[Int](vi, less_int);
  if i4 != 2 { io.println("int=" + i4.to_str()); return 4; }

  io.println("m308 ref-str-cmp value-args ok");
  return 0;
}
