// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-D07: Mutual recursive types -- type A points to B, type B points to A
type A = { val: Int; peer: *B; }
type B = { num: Int; buddy: *A; }

fn a_value(a: *A) -> Int {
  if a == (unsafe { 0 as *A }) { return 0; }
  var v: Int;
  unsafe { v = (*a).val; }
  return v;
}

fn b_value(b: *B) -> Int {
  if b == (unsafe { 0 as *B }) { return 0; }
  var v: Int;
  unsafe { v = (*b).num; }
  return v;
}

fn mutual_sum(a: *A) -> Int {
  if a == (unsafe { 0 as *A }) { return 0; }
  var val: Int;
  var b: *B;
  var aa: *A;
  unsafe { val = (*a).val; }
  unsafe { b = (*a).peer; }
  if b == (unsafe { 0 as *B }) { return val; }
  unsafe { aa = (*b).buddy; }
  return val + mutual_sum(aa);
}

fn main() -> Int {
  var ea: *A = unsafe { 0 as *A };
  var eb: *B = unsafe { 0 as *B };
  if a_value(ea) != 0 { return 1; }
  if b_value(eb) != 0 { return 2; }
  if mutual_sum(ea) != 0 { return 3; }
  return 0;
}
