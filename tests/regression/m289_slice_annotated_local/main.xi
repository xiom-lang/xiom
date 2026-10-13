// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m289 lock (REL-20261010-1548-stdlib-5): an ANNOTATED
// `let s: Slice[Int] = array.as_slice(&arr)` used to allocate an i64 slot
// holding field 0 (the data pointer) -- `concrete_type_for` stripped Slice
// to its element. `s.len()` then took the i64 -> Str.len path and strlen'd
// the data buffer ([1,2,3] printed 1) and the slice helpers AV'd on the
// annotated local. The canonical %struct.Slice slot must be used, and the
// by-ref option/result query matrix (the other filed shape) must read the
// live discriminator (stale-green on v0.64.4 already; kept pinned here).
module m289_slice_annotated_local.main

use xiom.core;
use xiom.array;

fn main() -> Int {
  var arr = [1, 2, 3];
  // Unannotated control (was already correct).
  let s0 = array.as_slice(&arr);
  if core.slice_len(&s0) != 3 { return 1; }
  // Annotated Slice[Int] local (the m289 fix).
  let s: Slice[Int] = array.as_slice(&arr);
  if s.len() != 3 { return 2; }
  if core.slice_len(&s) != 3 { return 3; }
  if core.is_sorted(&s) == false { return 4; }
  let f = core.slice_first(&s);
  if f.is_none { return 5; }
  let g = core.slice_get(&s, 1);
  if g.is_none { return 6; }
  let v = core.slice_to_vec(&s);
  if v.len() != 3 { return 7; }
  if v[2] != 3 { return 8; }
  // Generic by-ref Option/Result query matrix.
  var o: Option[Int] = Some(4);
  if core.option_is_some(&o) == false { return 9; }
  if core.option_is_none(&o) { return 10; }
  var o2: Option[Int] = None;
  if core.option_is_some(&o2) { return 11; }
  if core.option_is_none(&o2) == false { return 12; }
  var r: Result[Int, Str] = Ok(4);
  if core.result_is_ok(&r) == false { return 13; }
  if core.result_is_err(&r) { return 14; }
  var r2: Result[Int, Str] = Err("e");
  if core.result_is_ok(&r2) { return 15; }
  if core.result_is_err(&r2) == false { return 16; }
  return 0;
}
