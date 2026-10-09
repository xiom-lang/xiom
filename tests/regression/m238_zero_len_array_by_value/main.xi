// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m238 (wave-96 block 80): a ZERO-LENGTH fixed array passed BY VALUE to a
// generic `[N]T` param miscompiled at clang -- the param lowered to its
// element type (`i64`) while the call passed the `[0 x i64]` aggregate
// (error "'%tmp' defined with type '[0 x i64]' but expected 'i64'").
// The call now passes a zero element value (the callee can never index an
// empty array), and an annotated empty local records its element type so
// `[N]T` inference resolves T=Str for `[0]Str`.
module m238_zero_len_array_by_value

use xiom.array;

fn main() -> Int {
  // Explicit [0]Int annotation (the clang-mismatch shape); non-zero init
  // proves the init value survives the empty fold.
  let i0: [0]Int = [];
  var a = array.fold(i0, 43, fn(acc: Int, x: Int) -> Int { return acc + x; });
  if a != 43 { return 1; }

  // [0]Str: element type taken from the annotation (was defaulted to Int
  // and the [0 x i8*] arg clashed with the i64 param).
  let s0: [0]Str = [];
  var b = array.fold(s0, 44, fn(acc: Int, x: Str) -> Int { return acc; });
  if b != 44 { return 2; }

  // Untyped empty local + inline literal keep their existing paths.
  let u0 = [];
  var c = array.fold(u0, 45, fn(acc: Int, x: Int) -> Int { return acc + x; });
  if c != 45 { return 3; }
  var d = array.fold([], 46, fn(acc: Int, x: Int) -> Int { return acc + x; });
  if d != 46 { return 4; }

  // Non-empty controls.
  let i3 = [1, 2, 3];
  var e = array.fold(i3, 0, fn(acc: Int, x: Int) -> Int { return acc + x; });
  if e != 6 { return 5; }
  let s3 = ["a", "b", "c"];
  var f = array.fold(s3, 0, fn(acc: Int, x: Str) -> Int { return acc + 1; });
  if f != 3 { return 6; }

  return 0;
}
