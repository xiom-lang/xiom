// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X04: Combinatorial + Differential -- compute-then-return vs direct-return with struct
type Triple = { a: Int; b: Int; c: Int; }
fn compute_then_return(t: Triple) -> Int {
  var sum: Int = t.a + t.b + t.c;
  var prod: Int = t.a * t.b * t.c;
  return sum + prod;
}
fn direct_return(t: Triple) -> Int {
  return (t.a + t.b + t.c) + (t.a * t.b * t.c);
}
fn compare(t: Triple) -> Int {
  var a = compute_then_return(t);
  var b = direct_return(t);
  if a == b { return 0; }
  return 1;
}
fn main() -> Int {
  var t1 = Triple{ a: 2; b: 3; c: 4; };
  var t2 = Triple{ a: 5; b: 6; c: 7; };
  var t3 = Triple{ a: 1; b: 1; c: 1; };
  if compare(t1) == 0 && compare(t2) == 0 && compare(t3) == 0 { return 0; }
  return 1;
}
