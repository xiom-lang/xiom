// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m239 (queue item 20): `==`/`!=` on containers must compare CONTENT, not
// the erased field bits (data pointers / boxed payload handles). Covers
// Vec[Int] / Vec[Str] / Vec[Vec[Int]] / Vec[Option[Int]] / Vec[struct],
// Option[Vec] and Result payloads (including the packages res_eq probe),
// empty and non-empty, both == and !=.
module m239_deep_container_eq

type Pair = { a: Int; b: Str; }

fn main() -> Int {
  // ---- Vec[Int]: non-empty equal / unequal / length mismatch
  var p = Vec[Int].new();
  p.push(1); p.push(2); p.push(3);
  var q = Vec[Int].new();
  q.push(1); q.push(2); q.push(3);
  if p != q { return 1; }
  var r = Vec[Int].new();
  r.push(1); r.push(2); r.push(4);
  if p == r { return 2; }
  if !(p != r) { return 3; }
  var s = Vec[Int].new();
  s.push(1); s.push(2);
  if p == s { return 4; }

  // ---- empty vectors
  var e1 = Vec[Int].new();
  var e2 = Vec[Int].new();
  if e1 != e2 { return 5; }
  if e1 == p { return 6; }

  // ---- Vec[Str]: content equality across separate handles
  var sa = Vec[Str].new();
  sa.push("alpha"); sa.push("beta");
  var sb = Vec[Str].new();
  let a1 = "alpha";
  sb.push(a1); sb.push("beta");
  if sa != sb { return 7; }
  var sc = Vec[Str].new();
  sc.push("alpha"); sc.push("gamma");
  if sa == sc { return 8; }
  var se = Vec[Str].new();
  if se == sa { return 9; }

  // ---- nested Vec[Vec[Int]]
  var n1 = Vec[Vec[Int]].new();
  n1.push(p); n1.push(e1);
  var n2 = Vec[Vec[Int]].new();
  n2.push(q); n2.push(e2);
  if n1 != n2 { return 10; }
  var n3 = Vec[Vec[Int]].new();
  n3.push(q); n3.push(p);
  if n1 == n3 { return 11; }

  // ---- Vec[Option[Int]]
  var o1 = Vec[Option[Int]].new();
  o1.push(Some(7)); o1.push(None);
  var o2 = Vec[Option[Int]].new();
  o2.push(Some(7)); o2.push(None);
  if o1 != o2 { return 12; }
  var o3 = Vec[Option[Int]].new();
  o3.push(Some(8)); o3.push(None);
  if o1 == o3 { return 13; }
  var o4 = Vec[Option[Int]].new();
  o4.push(None);
  if o1 == o4 { return 14; }

  // ---- Result payloads
  var ra: Result[Int, Str] = Ok(5);
  var rb: Result[Int, Str] = Ok(5);
  if ra != rb { return 15; }
  var rc: Result[Int, Str] = Ok(6);
  if ra == rc { return 16; }
  var re: Result[Int, Str] = Err("bad");
  var rf: Result[Int, Str] = Err("bad");
  if re != rf { return 17; }
  var rg: Result[Int, Str] = Err("worse");
  if re == rg { return 18; }
  if ra == re { return 19; }

  // ---- packages res_eq acceptance: Ok(Vec) payload recursion
  var va: Result[Vec[UInt8], Int] = Ok(Vec[UInt8].new());
  var vb: Result[Vec[UInt8], Int] = Ok(Vec[UInt8].new());
  if va != vb { return 20; }

  // ---- Result[Vec[Int], Str]: non-empty payload recursion
  var rh: Result[Vec[Int], Str] = Ok(p);
  var ri: Result[Vec[Int], Str] = Ok(q);
  if rh != ri { return 21; }
  var rj: Result[Vec[Int], Str] = Ok(r);
  if rh == rj { return 22; }

  // ---- Option[Vec[Int]]
  var ov1: Option[Vec[Int]] = Some(p);
  var ov2: Option[Vec[Int]] = Some(q);
  if ov1 != ov2 { return 23; }
  var ov0: Option[Vec[Int]] = None;
  if ov1 == ov0 { return 24; }

  // ---- Vec[struct] content recursion (scalar + Str fields)
  var st1 = Vec[Pair].new();
  st1.push(Pair { a: 1, b: "x" });
  var st2 = Vec[Pair].new();
  let bx = "x";
  st2.push(Pair { a: 1, b: bx });
  if st1 != st2 { return 25; }
  var st3 = Vec[Pair].new();
  st3.push(Pair { a: 2, b: "x" });
  if st1 == st3 { return 26; }

  return 0;
}
