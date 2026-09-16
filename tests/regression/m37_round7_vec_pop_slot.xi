// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m37_round7_vec_pop_slot -- round-7 (2026-08-20) regression:
// inlined Vec.pop + match Option slot on an EMPTY vec -- the caller's match
// must read the inlined Option discriminant (the bare "pop" key resolved to
// nothing -> no scrutinee alloca -> unconditional Some arm -> exit 1).
// Also covers: Vec.first/last (mono'd -- were zero-param stubs), Vec.remove
// (pointer-arithmetic GEP scaling + element-width loads in mono'd bodies),
// and the *UInt8 buffer concat-gate (Vec[UInt8].first on an i8* buffer).
module m37_round7_vec_pop_slot
use xiom.collections;

fn main() -> Int {
  // 1. pop on an EMPTY vec must hit the None arm (round-7 ve2).
  var v = Vec[Int].new();
  match v.pop() {
    Some(_) => { return 1; }
    None => { }
  }
  // 2. get's None path (already worked -- keep pinned).
  match v.get(0) {
    Some(_) => { return 2; }
    None => { }
  }
  // 3. first/last on a populated vec (mono'd methods -- were zero-param stubs).
  v.push(42);
  v.push(99);
  match v.first() {
    Some(x) => { if x != 42 { return 3; } }
    None => { return 4; }
  }
  match v.last() {
    Some(x) => { if x != 99 { return 5; } }
    None => { return 6; }
  }
  // 4. first/last on an EMPTY vec must hit None.
  var w = Vec[Int].new();
  match w.first() {
    Some(_) => { return 7; }
    None => { }
  }
  match w.last() {
    Some(_) => { return 8; }
    None => { }
  }
  // 5. element scaling: values > 255 in mono'd pointer arithmetic
  //    (Vec[Int] data is i64*, GEP must scale by 8, loads must be i64).
  w.push(1000);
  w.push(2000);
  w.push(3000);
  match w.last() {
    Some(x) => { if x != 3000 { return 9; } }
    None => { return 10; }
  }
  // 6. Vec.remove -- mono'd body with element shift loop.
  match w.remove(0) {
    Some(x) => { if x != 1000 { return 11; } }
    None => { return 12; }
  }
  if w.len() != 2 { return 13; }
  // 7. Vec[UInt8] first/last -- i8* byte buffer must NOT hit the Str-concat
  //    intercept (`data + len` stays pointer arithmetic).
  var u = Vec[UInt8].new();
  u.push(10);
  u.push(20);
  u.push(30);
  match u.first() {
    Some(x) => { if x != 10 { return 14; } }
    None => { return 15; }
  }
  match u.last() {
    Some(x) => { if x != 30 { return 16; } }
    None => { return 17; }
  }
  // 8. Str + Int concat must still work (concat gate must not block it).
  var s = "y = " + 42;
  if s != "y = 42" { return 18; }
  return 0;
}
