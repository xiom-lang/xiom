// XIOM -- phase1_generics
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

fn wrap[T](x: T) -> T {
  return x;
}

fn main() -> Int {
  return wrap(42);
}
