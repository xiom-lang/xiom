// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T12: Generic constraints -- identity, struct, enum unwrap
fn id[T](x: T) -> T { return x; }
type Pair[T] = { first: T; second: T; }
enum Option[T] { Some(val: T), None }
fn generic_if[T](cond: Bool, a: T, b: T) -> T { if cond { return a; } return b; }
fn main() -> Int {
  if id(42) != 42 { return 1; }
  if id(true) != true { return 2; }
  var p: Pair[Int] = Pair{ first: 10; second: 20; };
  if p.first != 10 { return 3; }
  if generic_if(true, 100, 200) != 100 { return 4; }
  return 0;
}

