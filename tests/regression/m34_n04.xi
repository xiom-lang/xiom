// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N04: Generic struct derive[Eq] with Int instantiation
type Pair[T] = { first: T; second: T; } derive[Eq]
fn main() -> Int {
  var a = Pair[Int]{ first: 10; second: 20; };
  var b = Pair[Int]{ first: 10; second: 20; };
  var c = Pair[Int]{ first: 30; second: 40; };
  if a == b && a != c { return 0; }
  return 1;
}
