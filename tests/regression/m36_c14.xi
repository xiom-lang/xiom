// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C14: Every struct nesting depth 1-6 -- deeply nested structs with field access through all levels
type L1 = { a: Int; }
type L2 = { inner: L1; b: Int; }
type L3 = { inner: L2; c: Int; }
type L4 = { inner: L3; d: Int; }
type L5 = { inner: L4; e: Int; }
type L6 = { inner: L5; f: Int; }
fn depth1_access(s: L1) -> Int { return s.a; }
fn depth2_access(s: L2) -> Int { return s.inner.a + s.b; }
fn depth3_access(s: L3) -> Int { return s.inner.inner.a + s.inner.b + s.c; }
fn depth4_access(s: L4) -> Int { return s.inner.inner.inner.a + s.inner.inner.b + s.inner.c + s.d; }
fn depth5_access(s: L5) -> Int { return s.inner.inner.inner.inner.a + s.inner.inner.inner.b + s.inner.inner.c + s.inner.d + s.e; }
fn depth6_access(s: L6) -> Int { return s.inner.inner.inner.inner.inner.a + s.inner.inner.inner.inner.b + s.inner.inner.inner.c + s.inner.inner.d + s.inner.e + s.f; }
fn main() -> Int {
  var d1 = L1{ a: 1; };
  if depth1_access(d1) != 1 { return 1; }
  var d2 = L2{ inner: d1; b: 2; };
  if d2.b != 2 { return 2; }
  if depth2_access(d2) != 3 { return 3; }
  var d3 = L3{ inner: d2; c: 3; };
  if depth3_access(d3) != 6 { return 4; }
  var d4 = L4{ inner: d3; d: 4; };
  if depth4_access(d4) != 10 { return 5; }
  var d5 = L5{ inner: d4; e: 5; };
  if depth5_access(d5) != 15 { return 6; }
  var d6 = L6{ inner: d5; f: 6; };
  if depth6_access(d6) != 21 { return 7; }
  return 0;
}
