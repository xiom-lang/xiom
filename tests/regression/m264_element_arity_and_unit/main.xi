// m264 (REL-20261011-1121 B-14/B-15, 2026-10-11): checker + storage locks.
//   * B-15: a Vec[fn] ELEMENT call with >= 2 arguments used as a VALUE
//     (`if h(a, b)` / `let got = h(a, b)`) must type the fn's declared
//     return -- the element type "fn(Int, Int) -> Bool" was COMMA-SPLIT
//     ("fn(Int" + "Int) -> Bool") so the call typed as `()`.
//   * B-14: a bare `return;` inside an explicitly `-> Unit` fn must
//     unify Unit/"()".
//   * Storage: pushing a bare fn NAME into a MODULE-GLOBAL Vec[fn] must
//     store the closure ENV (the element-call path loads field 0; a raw
//     code address deref'd the machine code -> 0xC0000005).
// Exit 0 on success.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m264_element_arity_and_unit

var h1: Vec[fn(Int, Int) -> Bool] = Vec[fn(Int, Int) -> Bool].new();

fn f_eq(a: Int, b: Int) -> Bool {
  return a == b;
}

// B-15 shape: element call with two arguments, result used in `if`.
fn d1(a: Int, b: Int) -> Bool {
  if h1.len() == 0 {
    return false;
  }
  let h = h1[h1.len() - 1];
  if h(a, b) {
    return true;
  }
  return false;
}

// B-15 shape: element call result bound with `let`.
fn d2(a: Int, b: Int) -> Bool {
  let got = h1[0](a, b);
  return got;
}

// B-14 shape: bare `return;` in an explicitly `-> Unit` fn.
fn early(x: Int) -> Unit {
  if x == 0 {
    return;
  }
}

fn main() -> Int {
  h1.push(f_eq);
  var r = 0;
  if d1(1, 1) {
    r = r + 1;
  }
  if d1(1, 2) {
    r = r + 100;
  }
  if d2(3, 3) {
    r = r + 10;
  }
  early(0);
  early(1);
  return r - 11;
}
