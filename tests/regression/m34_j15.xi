// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J15: Module with generic type -- generic functions and types in modules
module generic {
  pub fn ident[T](x: T) -> T { return x; }
  pub fn add[T](a: T, b: T) -> T { return a + b; }
  pub type Pair[T] = { first: T; second: T; }
}
use generic.ident;
use generic.add;
use generic.Pair;
fn main() -> Int {
  var r1 = ident[Int](42);
  if r1 != 42 { return 1; }
  var r2 = add[Int](30, 12);
  if r2 != 42 { return 2; }
  var p = Pair[Int]{ first: 10, second: 20 };
  if p.first != 10 { return 3; }
  if p.second != 20 { return 4; }
  var pb = Pair[Bool]{ first: true, second: false };
  if pb.first != true { return 5; }
  return 0;
}
