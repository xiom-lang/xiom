// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m46_round14_vec_str_elems_narrow -- round-14 (2026-08-22) regression:
// (1) method calls on Vec[Str] ELEMENTS (`v[0].len()`, `h.names[i].
// starts_with(..)`) emitted an invalid GEP (getelementptr i8*, i8**, 0, 1)
// -- the elem-load switch yields i64, so the receiver wasn't recognized
// as a string and the generic dispatch GEP'd the i8* as a struct. The
// Str builtin paths (len/slice/substr/starts_with/ends_with) now detect
// Vec[Str]-element receivers (Ident and struct-FIELD containers).
// (2) narrow-SIGNED Vec loads (pop/get/index on Int16/Int8) zext'd the
// bit pattern (-30000 -> 35536) -- emit_elem_load now sign-extends when
// the container's element type is signed; unsigned stays zext.
module m46_round14_vec_str_elems_narrow
use xiom.core;
use xiom.string;

type Holder = {
  names: Vec[Str];
}

fn total_len(v: &Vec[Str]) -> Int {
  var sum = 0;
  var i = 0;
  while i < v.len() {
    sum = sum + v[i].len();
    i = i + 1;
  }
  return sum;
}

fn main() -> Int {
  // Vec[Str] element len on a local
  var v = Vec[Str].new();
  v.push("hello");
  v.push("world");
  if v[0].len() != 5 { return 1; }
  // Vec[Str] element starts_with
  if !v[1].starts_with("wor") { return 2; }
  // &Vec[Str] param element len in a loop
  if total_len(&v) != 10 { return 3; }
  // struct-FIELD Vec[Str] element len
  var h = Holder{ names: Vec[Str].new(); };
  h.names.push("hello");
  if h.names[0].len() != 5 { return 4; }
  // narrow-SIGNED loads: Int16 -30000 via push/pop/get/index
  var n = Vec[Int16].new();
  n.push(-30000);
  n.push(7);
  match n.pop() {
    Some(x) => { if x != 7 { return 5; } },
    None => { return 6; },
  };
  match n.get(0) {
    Some(x) => { if x != -30000 { return 7; } },
    None => { return 8; },
  };
  if n[0] != -30000 { return 9; }
  // Int8 signed + UInt8 stays unsigned
  var w = Vec[Int8].new();
  w.push(-5);
  match w.pop() {
    Some(x) => { if x != -5 { return 10; } },
    None => { return 11; },
  };
  var u = Vec[UInt8].new();
  u.push(200);
  match u.pop() {
    Some(x) => { if x != 200 { return 12; } },
    None => { return 13; },
  };
  return 0;
}
