// m293 (REL-20261010-1549-packages-11): deep CONTENT equality for the
// Map/Set shells. Pre-fix, `m1 == m2` on `Map`/`Set`/legacy `HashSet`
// values fell into the struct-equality cascade and emitted the invalid
// `icmp eq %struct.Vec` for the by-value keys/values/items fields (clang:
// "icmp requires integer operands"). The lock covers:
// - Map[Int, Int] / Set[Int]: concrete-arg arm over the keys/values/items
//   Vecs (equal, differing value, length mismatch, `!=`);
// - Map[Str, Int] / Set[Str] with DISTINCT runtime buffers for equal
//   contents (the element compare must strcmp, not compare pointers);
// - a clause site (`ensures: result == (a == b)`) for the params form;
// - BTreeMap[Int, Int] / BTreeSet[Int] (same keys/values/items shapes);
// - the legacy non-generic `xiom.collect.hashset.HashSet` with PERMUTED
//   insertion order (same final probe layout for these values) plus a
//   size-changing insert.
// SAME-ORDER caveat: the compare is layout-based; equal contents whose
// open-addressing layouts differ under probe collisions need lookup-based
// semantics (separate follow-up).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m293_mapset_eq;

use xiom.collections;
use xiom.collect.hashset;
use xiom.convert;

fn map_eq_clause(a: Map[Int, Int], b: Map[Int, Int]) -> Bool
  ensures: result == (a == b)
{
  return a == b;
}

fn main() -> Int {
  // Map[Int, Int]
  var m1 = Map[Int, Int].new();
  m1.insert(1, 10);
  m1.insert(2, 20);
  var m2 = Map[Int, Int].new();
  m2.insert(1, 10);
  m2.insert(2, 20);
  var m3 = Map[Int, Int].new();
  m3.insert(1, 10);
  m3.insert(2, 99);
  var m4 = Map[Int, Int].new();
  m4.insert(1, 10);
  if !(m1 == m2) { return 3; }
  if m1 != m2 { return 4; }
  if m1 == m3 { return 5; }
  if !(m1 != m3) { return 6; }
  if m1 == m4 { return 7; }
  if !(m1 != m4) { return 8; }
  if !map_eq_clause(m1, m2) { return 9; }
  if map_eq_clause(m1, m3) { return 10; }

  // Map[Str, Int]: strcmp keys with DISTINCT buffers
  var ka = convert.int_to_string(7);
  var kb = convert.int_to_string(7);
  var m5 = Map[Str, Int].new();
  m5.insert(ka, 1);
  m5.insert("x", 2);
  var m6 = Map[Str, Int].new();
  m6.insert(kb, 1);
  m6.insert("x", 2);
  var m7 = Map[Str, Int].new();
  m7.insert(kb, 1);
  m7.insert("x", 3);
  if !(m5 == m6) { return 11; }
  if m5 == m7 { return 12; }
  if !(m5 != m7) { return 13; }

  // Set[Int]
  var s1 = Set[Int].new();
  s1.insert(1);
  s1.insert(2);
  s1.insert(3);
  var s2 = Set[Int].new();
  s2.insert(1);
  s2.insert(2);
  s2.insert(3);
  var s3 = Set[Int].new();
  s3.insert(1);
  s3.insert(2);
  if !(s1 == s2) { return 14; }
  if s1 == s3 { return 15; }
  if !(s1 != s3) { return 16; }

  // Set[Str] same-order, distinct buffers
  var sa = convert.int_to_string(42);
  var sb = convert.int_to_string(42);
  var s4 = Set[Str].new();
  s4.insert(sa);
  var s5 = Set[Str].new();
  s5.insert(sb);
  if !(s4 == s5) { return 17; }
  var sc = convert.int_to_string(43);
  var s6 = Set[Str].new();
  s6.insert(sc);
  if s4 == s6 { return 18; }

  // BTreeMap[Int, Int]
  var b1 = BTreeMap[Int, Int].new();
  b1.insert(1, 10);
  b1.insert(2, 20);
  var b2 = BTreeMap[Int, Int].new();
  b2.insert(1, 10);
  b2.insert(2, 20);
  var b3 = BTreeMap[Int, Int].new();
  b3.insert(1, 10);
  if !(b1 == b2) { return 19; }
  if b1 == b3 { return 20; }

  // BTreeSet[Int]
  var t1 = BTreeSet[Int].new();
  t1.insert(5);
  t1.insert(6);
  var t2 = BTreeSet[Int].new();
  t2.insert(5);
  t2.insert(6);
  var t3 = BTreeSet[Int].new();
  t3.insert(5);
  if !(t1 == t2) { return 21; }
  if t1 == t3 { return 22; }

  // collect HashSet: permuted insertion order, same final layout for these values
  var h1 = hashset_new();
  var h2 = hashset_new();
  hashset_insert(&mut h1, 7);
  hashset_insert(&mut h1, 9);
  hashset_insert(&mut h2, 9);
  hashset_insert(&mut h2, 7);
  if !(h1 == h2) { return 23; }
  hashset_insert(&mut h2, 11);
  if h1 == h2 { return 24; }

  return 0;
}
