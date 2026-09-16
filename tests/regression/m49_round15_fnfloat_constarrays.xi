// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m49_round15_fnfloat_constarrays -- round-15 (2026-08-23) regressions:
// (1) fn-typed params returning/accepting Float64 marshal through the
//     __fnwrap thunk with REAL double types (the old i64 declaration
//     read the wrong register class + sitofp'd the bit pattern --
//     apply(sqminus2, 2.0) returned 0; smoke_math_numerical/analysis/
//     calculus/integral/optimization were all blocked);
// (2) const-generic [N]T arrays: array.len re-publishes N per distinct
//     argument array (len(&[7,8]) = 2 THEN len(&[1,2,3,4]) = 4 -- the
//     old const map cached the first call's N on the shared mono name);
//     narrow-element reads through &[N]T mono bodies (array.first on
//     [1 as Int8, 2, 3] returned 0 -- the caller's array local leaked
//     into the mono body and the +1 array-buffer path fired; UInt8
//     elements zero-extend: 200, not -56);
// (3) array.map's [N]U result resolves implicitly (map(arr, fn) then
//     array.len(&doubled) == 5 -- the by-value [N]T param now resolves
//     to "[5 x i64]" and the caller materializes the aggregate from the
//     Vec conversion);
// (4) NON-pub catalog interfaces inject (core.xi's `interface Ord[T]`
//     was skipped by the is_pub gate -- `Ord[T].compare(a, b)` inside
//     mono'd stdlib bodies misread as a value-instance method with a
//     literal-0 receiver; BinaryHeap popped 2,3,1,5 instead of 5,3,2,1).
module m49_round15_fnfloat_constarrays
use xiom.array;
use xiom.core;

fn sqminus2(x: Float64) -> Float64 {
  return x * x - 2.0;
}

fn apply(f: fn(Float64) -> Float64, x: Float64) -> Float64 {
  f(x)
}

fn apply2(f: fn(Float64) -> Float64, g: fn(Int) -> Int, x: Float64, n: Int) -> Float64 {
  var a = f(x);
  var b = g(n) as Float64;
  return a + b;
}

fn plus10(n: Int) -> Int {
  return n + 10;
}

fn main() -> Int {
  // 1. fn-ref Float64 through the env-first thunk.
  var r = apply(sqminus2, 2.0);
  if r != 2.0 { return 1; }
  var r2 = apply2(sqminus2, plus10, 2.0, 3);
  if r2 != 15.0 { return 2; }
  // 1b. closure literal with Float64 param + return.
  var c1 = fn(x: Float64) -> Float64 { return x * 3.0; };
  var r3 = apply(c1, 5.0);
  if r3 != 15.0 { return 3; }

  // 2. const-N: len must re-publish per array.
  var a2 = [7, 8];
  if array.len(&a2) != 2 { return 10; }
  var a4 = [1, 2, 3, 4];
  if array.len(&a4) != 4 { return 11; }
  var a1 = [9];
  if array.len(&a1) != 1 { return 12; }
  // 2b. narrow-element reads.
  var arr8 = [1 as Int8, 2 as Int8, 3 as Int8];
  match array.first(&arr8) {
    Some(v) => { if v != 1 as Int8 { return 13; } },
    None => { return 14; },
  };
  match array.get(&arr8, 1) {
    Some(v) => { if v != 2 as Int8 { return 15; } },
    None => { return 16; },
  };
  var bu = [200 as UInt8, 100 as UInt8];
  match array.first(&bu) {
    Some(v) => { if v != 200 as UInt8 { return 17; } },
    None => { return 18; },
  };

  // 3. array.map implicit [N]U result + len on it.
  var arr = [1, 2, 3, 4, 5];
  var doubled = array.map(arr, fn(x: Int) -> Int { return x * 2; });
  if array.len(&doubled) != 5 { return 20; }
  if doubled[0] != 2 || doubled[4] != 10 { return 21; }

  // 4. Ord[T].compare dispatch in mono'd stdlib bodies (max-heap order).
  var h = BinaryHeap[Int].new();
  h.push(3);
  h.push(1);
  h.push(5);
  h.push(2);
  var prev = 1000;
  var i = 0;
  while i < 4 {
    match h.pop() {
      Some(v) => {
        if v > prev { return 30; }
        prev = v;
      },
      None => { return 31; },
    }
    i = i + 1;
  }
  if prev != 1 { return 32; }

  return 0;
}
