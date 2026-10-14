// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m308 lock (REL-20261010-1553-playground-2 follow-up): a by-reference
// comparator parameter (`less: fn(&Str, &Str) -> Bool`) must compare string
// CONTENT. Pre-fix the m286 strcmp branch only matched plain "Str" locals, so
// `a < b` on `&Str` params fell to the pointer auto-deref and compared the
// first byte of each char pointer: find_min on ["pear","cherry","apple"]
// returned index 0 instead of 2. The explicit-deref (`*a < *b`) and Int
// controls pin the fix to the implicit-deref Str shape.
module m308_ref_str_cmp.main

use xiom.io;

fn less_impl(a: &Str, b: &Str) -> Bool { return a < b; }
fn less_expl(a: &Str, b: &Str) -> Bool { return *a < *b; }
fn less_int_impl(a: &Int, b: &Int) -> Bool { return a < b; }

fn find_min_g[T](arr: &Vec[T], less: fn(&T, &T) -> Bool) -> Int
{
  var best: Int = 0;
  var i: Int = 1;
  while i < arr.len() {
    if less(&arr[i], &arr[best]) { best = i; }
    i = i + 1;
  }
  return best;
}

fn main() -> Int
{
  var vs: Vec[Str] = Vec[Str].new();
  vs.push("pear");
  vs.push("cherry");
  vs.push("apple");

  let i1 = find_min_g[Str](&vs, less_impl);
  if i1 != 2 { io.println("impl=" + i1.to_str()); return 1; }

  let i2 = find_min_g[Str](&vs, less_expl);
  if i2 != 2 { io.println("expl=" + i2.to_str()); return 2; }

  var vi: Vec[Int] = Vec[Int].new();
  vi.push(5);
  vi.push(12);
  vi.push(3);
  let i3 = find_min_g[Int](&vi, less_int_impl);
  if i3 != 2 { io.println("intimpl=" + i3.to_str()); return 3; }

  io.println("m308 ref-str-cmp ok");
  return 0;
}
