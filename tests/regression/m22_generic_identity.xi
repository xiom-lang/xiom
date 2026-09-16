// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: Generic identity and composition
fn id[T](x: T) -> T { return x; }
fn compose[T](a: T, b: T) -> Int { return 0; }
fn main() -> Int {
  var i: Int = id(42);
  if i == 42 {
    return 0;
  }
  return 1;
}
