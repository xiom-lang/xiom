// XIOM -- phase1_generics
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

fn wrap[T](x: T) -> T {
  return x;
}

fn main() -> Int {
  return wrap(42);
}
