// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// R10 regression: Vec[Option[struct-with-Str]] element reads.
// `c.groups[i]` on a `Vec[Option[M2]]` field memcpy'd an OPAQUE
// %struct.Option (16 bytes) from a 32-byte Option__M2 slot and unboxed the
// tag as a pointer (AV 0x10). Two roots: type_from_ast_with_args dropped
// nested Option/Result args in struct fields, and the field-element scan
// broke on the first matching type_meta key.
//
// NOTE: C2 is passed BY VALUE; each C2 instance is moved exactly once
// (build a fresh vector per read keeps the fixture a well-formed program).
module m65_vec_option_struct_str

type M2 = {
  start: Int;
  end: Int;
  text: Str;
}

type C2 = {
  groups: Vec[Option[M2]];
}

fn get2(c: C2, i: Int) -> Option[M2] {
  return c.groups[i];
}

fn main() -> Int {
  // First vector: three slots, read index 0.
  var g = Vec[Option[M2]].new();
  var m0 = M2{ start: 0; end: 1; text: "hi"; };
  g.push(Some(m0));
  var m1 = M2{ start: 4; end: 9; text: "yo"; };
  g.push(Some(m1));
  g.push(None);
  match get2(C2{ groups: g }, 0) {
    Some(x) => {
      if x.text != "hi" { return 1; }
      if x.start != 0 { return 2; }
      if x.end != 1 { return 3; }
    },
    None => { return 4; },
  }
  // Second vector: read index 1 (stride/positioning).
  var g2 = Vec[Option[M2]].new();
  var n0 = M2{ start: 7; end: 8; text: "aa"; };
  g2.push(Some(n0));
  var n1 = M2{ start: 10; end: 11; text: "bb"; };
  g2.push(Some(n1));
  match get2(C2{ groups: g2 }, 1) {
    Some(y) => {
      if y.text != "bb" { return 5; }
      if y.start != 10 { return 6; }
      if y.end != 11 { return 7; }
    },
    None => { return 8; },
  }
  // Third vector: a None slot only.
  var g3 = Vec[Option[M2]].new();
  g3.push(None);
  match get2(C2{ groups: g3 }, 0) {
    Some(_) => { return 9; },
    None => {},
  }
  return 0;
}
