// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N12: Generic struct derive[Eq] with Int -- equality on generic type
type Container[T] = { data: T; tag: Int; } derive[Eq]
fn main() -> Int {
  var a = Container[Int]{ data: 42; tag: 1; };
  var b = Container[Int]{ data: 42; tag: 1; };
  var c = Container[Int]{ data: 99; tag: 1; };
  var d = Container[Int]{ data: 42; tag: 2; };
  if a == b && a != c && a != d { return 0; }
  return 1;
}
