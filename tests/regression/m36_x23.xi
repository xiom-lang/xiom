// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X23: Deep generics -- nested generic type parameters and compositions
fn compose[A, B, C](f: fn(B) -> C, g: fn(A) -> B, x: A) -> C {
  return f(g(x));
}
fn id[A](x: A) -> A { return x; }
fn double(x: Int) -> Int { return x * 2; }
fn triple(x: Int) -> Int { return x * 3; }
fn add_one(x: Int) -> Int { return x + 1; }
fn make_pair[A, B](a: A, b: B) -> Int { return 0; }
fn swap_order[A, B](a: A, b: B) -> Int { return 0; }
fn nested_generic[A](x: A, y: A) -> A {
  if x > y { return x; }
  return y;
}
fn deep_chain[A](a: A, b: A, c: A) -> A {
  var m1 = nested_generic[A](a, b);
  return nested_generic[A](m1, c);
}
fn main() -> Int {
  if id[Int](42) != 42 { return 1; }
  if double(triple(3)) != 18 { return 2; }
  if add_one(double(5)) != 11 { return 3; }
  if deep_chain[Int](1, 5, 3) != 5 { return 4; }
  if deep_chain[Int](10, 2, 7) != 10 { return 5; }
  return 0;
}
