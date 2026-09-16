// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// LET-array decision P3 regression (docs/LET_ARRAY_DECISION.md): an
// UNANNOTATED `let a = [...]` binds a FIXED array `[N]T` (the M33 let->Vec
// conversion is gone for let literals). Call-site compatibility is preserved
// by materializing a heap-backed Vec with the array's elements (len = N,
// cap = max(N,16), elem_size = size_of(T)) whenever the array is passed to a
// by-value `%struct.Vec` param (`&Slice[T]`/`Vec[T]`). The elements are
// copied, not aliased: by-value Vec params may push (a stack view corrupted
// the heap in test_algo's concat). `a.len()` is now the const N (pre-P3 it
// read the Vec header; on a fixed array it fell to a 0 stub). Non-generic
// `&Slice[Int]` params (`core.sum_slice`) now use the same by-value
// %struct.Vec ABI as the generic ones -- the old element-pointer lowering
// read data[0] as the length. Pre-P3 this fixture exits 8 (sum_slice = 0).
module m69_let_array_slice_bridge
use xiom.array;
use xiom.core;

fn main() -> Int {
  let a = [1, 2, 3, 4, 5];
  if a.len() != 5 { return 1; }
  if a[4] != 5 { return 2; }
  if array.len(&a) != 5 { return 3; }
  if !core.is_sorted(a) { return 4; }
  if !core.contains(a, 3) { return 5; }
  if core.contains(a, 99) { return 6; }
  if !core.all(a, fn(x: Int) -> Bool { return x > 0; }) { return 7; }
  if core.sum_slice(a) != 15 { return 8; }
  let f = array.first(&a);
  match f {
    Some(v) => { if v != 1 { return 9; } };
    None => { return 10; };
  };
  var s = array.as_slice(&a);
  if s.len() != 5 { return 11; }
  let g: [3]Float64 = [1.5, 2.5, 3.5];
  if g[2] != 3.5 { return 12; }
  let n = [1 as Int8, 2 as Int8, 3 as Int8];
  if n.len() != 3 { return 13; }
  if n[2] != 3 { return 14; }
  return 0;
}
