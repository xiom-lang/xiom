// XIOM -- phase1_contracts
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

type PositiveInt = {
  value: Int;
  invariant: value > 0;
}

fn divide(a: Float64, b: Float64) -> Float64
  requires: b != 0.0
  ensures: result * b == a
{
  return a / b;
}

fn main() -> Int {
  let x = divide(10.0, 2.0);
  return 0;
}
