// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E20: Chained field access -- 10-level nested struct field reads
type N1 = { f: Int; }
type N2 = { f: Int; }
type N3 = { f: Int; }
type N4 = { f: Int; }
type N5 = { f: Int; }
type N6 = { f: Int; }
type N7 = { f: Int; }
type N8 = { f: Int; }
type N9 = { f: Int; }
type N10 = { f: Int; }
fn main() -> Int {
  var n1 = N1{ f: 0 };
  var n2 = N2{ f: n1.f + 1 };
  var n3 = N3{ f: n2.f + 1 };
  var n4 = N4{ f: n3.f + 1 };
  var n5 = N5{ f: n4.f + 1 };
  var n6 = N6{ f: n5.f + 1 };
  var n7 = N7{ f: n6.f + 1 };
  var n8 = N8{ f: n7.f + 1 };
  var n9 = N9{ f: n8.f + 1 };
  var n10 = N10{ f: n9.f + 1 };
  if n10.f != 9 { return 1; }
  return 0;
}