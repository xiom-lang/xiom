// XIOM -- phase1_generics
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn wrap[T](x: T) -> T {
  return x;
}

fn main() -> Int {
  return wrap(42);
}
